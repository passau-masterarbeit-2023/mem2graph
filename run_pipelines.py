import os
import subprocess
import threading
import tqdm
import sys
from datetime import datetime
import threading

INPUT_FILE_DIR_PATH = "/home/onyr/code/phdtrack/phdtrack_data_clean"
COMPUTE_INSTANCE_TIMEOUT = 0 # timeout in seconds

# -------------------- possible pipelines (filtering) -------------------- #
PIPELINES_NAMES_TO_ADDITIONAL_ARGS_FILTER: list[tuple[str, list[str]]] = [
    ("value-node-embedding", []),
    ("chunk-top-vn-semantic-embedding", []),
    ("chunk-semantic-embedding", ["-v", "-a", "chunk-header-node"]),
    ("chunk-semantic-embedding", ["-a", "chunk-header-node"]),
    ("chunk-statistic-embedding", ["-a", "chunk-header-node"]),
    ("chunk-start-bytes-embedding", ["-a", "chunk-header-node"]),
    ("chunk-extraction", ["-a", "chunk-header-node"]),
    # ("graph-with-embedding-comments", ["-v", "-a", "chunk-header-node", "-c", "chunk-semantic-embedding"]),
    # ("graph-with-embedding-comments", ["-v", "-a", "chunk-header-node", "-c", "chunk-statistic-embedding"]),
    # ("graph-with-embedding-comments", ["-v", "-a", "chunk-header-node", "-c", "chunk-start-bytes-embedding"]),
]

# -------------------- possible pipelines (no filtering) -------------------- #
PIPELINES_NAMES_TO_ADDITIONAL_ARGS_NO_FILTER: list[tuple[str, list[str]]] = [
    ("graph", []),
    ("graph", ["-a", "none"]),
    ("graph", ["-v", "-a", "chunk-header-node"]),
    ("graph", ["-v", "-a", "none"]),
    # setting manually filtering, so as to limit the number of combinations
    ("graph-with-embedding-comments", ["-v", "-a", "chunk-header-node", "-c", "chunk-semantic-embedding", "-e", "none", "-s", "none"]),
    ("graph-with-embedding-comments", ["-v", "-a", "chunk-header-node", "-c", "chunk-statistic-embedding", "-e", "none", "-s", "none"]),
    ("graph-with-embedding-comments", ["-v", "-a", "chunk-header-node", "-c", "chunk-start-bytes-embedding", "-e", "none", "-s", "none"]),
    ("graph-with-embedding-comments", ["-v", "-a", "chunk-header-node", "-c", "chunk-semantic-embedding", "-e", "only-max-entropy", "-s", "activate"]),
    ("graph-with-embedding-comments", ["-v", "-a", "chunk-header-node", "-c", "chunk-statistic-embedding", "-e", "only-max-entropy", "-s", "activate"]),
    ("graph-with-embedding-comments", ["-v", "-a", "chunk-header-node", "-c", "chunk-start-bytes-embedding", "-e", "only-max-entropy", "-s", "activate"]),
]

PIPELINE_NAMES: set[str] = set()
for (pipeline_name, _) in PIPELINES_NAMES_TO_ADDITIONAL_ARGS_FILTER:
    PIPELINE_NAMES.add(pipeline_name)
for (pipeline_name, _) in PIPELINES_NAMES_TO_ADDITIONAL_ARGS_NO_FILTER:
    PIPELINE_NAMES.add(pipeline_name)

LIST_ENTROPY_FILTERING_FLAGS = [
    "none",
    "only-max-entropy",
    "min-of-chunk-treshold-entropy",
]

LIST_BYTE_SIZE_FILTERING_FLAGS = [
    "none",
    "activate",
]

# -------------------- CLI arguments -------------------- #
import sys
import argparse

# wrapped program flags
class CLIArguments:
    args: argparse.Namespace

    def __init__(self) -> None:
        self.__log_raw_argv()
        self.__parse_argv()
    
    def __log_raw_argv(self) -> None:
        print("Passed program params:")
        for i in range(len(sys.argv)):
            print("param[{0}]: {1}".format(
                i, sys.argv[i]
            ))
    
    def __parse_argv(self) -> None:
        """
        python main [ARGUMENTS ...]
        """
        parser = argparse.ArgumentParser(description='Program [ARGUMENTS]')
        parser.add_argument(
            '--dry-run',
            action='store_true',
            help="Run in dry mode, without running commands."
        )
        # no delete old output files
        parser.add_argument(
            '-k',
            '--keep-old-output',
            action='store_true',
            help="Keep old output files."
        )
        # add file path or directory path argument
        parser.add_argument(
            '-i',
            '--input',
            type=str,
            help="Input as file path or directory path"
        )
        # select only a list of selected compute instance
        parser.add_argument(
            '-r',
            '--run-selected',
            type=int,
            nargs='+',
            help="List of one or more integers."
        )
        # timeout for each compute instance
        parser.add_argument(
            '-t',
            '--timeout',
            type=int,
            help="Timeout in seconds for each compute instance."
        )
        # Only pipelines with selected name
        parser.add_argument(
            '-p',
            '--pipeline',
            type=str,
            help=f"Consider only pipelines with selected name. Possible values: [{', '.join(PIPELINE_NAMES)}]"
        )

        # save parsed arguments
        self.args = parser.parse_args()

        # overwrite debug flag
        global DRY_RUN
        DRY_RUN = True if self.args.dry_run else False

        global COMPUTE_INSTANCE_TIMEOUT
        if self.args.timeout is not None:
            COMPUTE_INSTANCE_TIMEOUT = self.args.timeout
            print(f"🔷 Compute instance timeout: {COMPUTE_INSTANCE_TIMEOUT} seconds")

        # overwrite input dir path
        global INPUT_FILE_DIR_PATH
        if self.args.input is not None:
            INPUT_FILE_DIR_PATH = self.args.input
        
        # check input dir path
        if not os.path.exists(INPUT_FILE_DIR_PATH):
            print(f"🔴 Input path {INPUT_FILE_DIR_PATH} does not exist. Abort processing.")
            exit()
        
        # pipeline name filtering
        if self.args.pipeline is not None:
            global PIPELINES_NAMES_TO_ADDITIONAL_ARGS_FILTER
            global PIPELINES_NAMES_TO_ADDITIONAL_ARGS_NO_FILTER
            PIPELINES_NAMES_TO_ADDITIONAL_ARGS_FILTER = [
                (name, args) for (name, args) in PIPELINES_NAMES_TO_ADDITIONAL_ARGS_FILTER if name == self.args.pipeline
            ]
            PIPELINES_NAMES_TO_ADDITIONAL_ARGS_NO_FILTER = [
                (name, args) for (name, args) in PIPELINES_NAMES_TO_ADDITIONAL_ARGS_NO_FILTER if name == self.args.pipeline
            ]
            print(f"🔷 Pipeline name filtering: {self.args.pipeline}")

        # log parsed arguments
        print("Parsed program params:")
        for arg in vars(self.args):
            print("{0}: {1}".format(
                arg, getattr(self.args, arg)
            ))

def create_or_clear_output_dir(output_dir_path: str, remove_old_files: bool) -> None:
    """
    Create the output dir if it does not exist.
    If it exists, remove all ".csv" and ".gv" files, if remove_old_files is True.
    """
    # create output dir if it does not exist
    if not os.path.exists(output_dir_path):
        os.makedirs(output_dir_path)
    else:
        if remove_old_files:
            # remove all ".csv" an ".gv" files in the output dir
            for filename in os.listdir(output_dir_path):
                if filename.endswith(".csv") or filename.endswith(".gv"):
                    os.remove(os.path.join(output_dir_path, filename))
                    print(f" 󰆴 -> Removed {filename} in {output_dir_path}")

def build_arg_compute_instances(cli: CLIArguments) -> list[list[str]]:
    """
    Create a list of CLI commands with arguments to run the executables.
    """

    # create a list of arguments to run the executables
    arg_compute_instances: list[list[str]] = []

    # create args with entropy
    for (pipeline_name, additional_agrs) in PIPELINES_NAMES_TO_ADDITIONAL_ARGS_FILTER:
        for entropy_filtering_flag in LIST_ENTROPY_FILTERING_FLAGS:  
            for byte_size_filter in LIST_BYTE_SIZE_FILTERING_FLAGS:    
                # output dir preparation
                current_dir = os.getcwd()

                # add the filtering if it is not none (easy to activate or not the sampling)
                filtering_tag = ""
                if entropy_filtering_flag != "none" or byte_size_filter != "none":
                    filtering_tag = "filtered_"
                    

                compute_instance_index = len(arg_compute_instances)
                output_dir_path = (
                    current_dir + "/data/" + str(compute_instance_index) + 
                    "_" + filtering_tag + 
                    pipeline_name.replace("-", "_") + 
                    "_" + "_".join(additional_agrs) +
                    "_-e_" + entropy_filtering_flag +
                    "_-s_" + byte_size_filter
                )
                
                create_or_clear_output_dir(output_dir_path, not cli.args.keep_old_output)
                
                # prepare arguments
                args = [
                    "cargo", "run", "--",
                    "-d", INPUT_FILE_DIR_PATH,
                    "-o", output_dir_path, 
                    "-p", pipeline_name,
                    "-e", entropy_filtering_flag,
                    "-s", byte_size_filter,
                ]

                # append additional arguments
                if len(additional_agrs) > 0:
                    args.extend(additional_agrs)

                arg_compute_instances.append(args)
        
    # create args without entropy
    for (pipeline_name, additional_agrs) in PIPELINES_NAMES_TO_ADDITIONAL_ARGS_NO_FILTER:
        # output dir preparation
        current_dir = os.getcwd()
        additional_param_list_as_str = "_".join(
            additional_agrs
        )
        compute_instance_index = len(arg_compute_instances)
        output_dir_path = current_dir + "/data/" + str(compute_instance_index) + "_" + pipeline_name.replace("-", "_") + "_-e_none_" + additional_param_list_as_str
        
        create_or_clear_output_dir(output_dir_path, not cli.args.keep_old_output)
        
        # prepare arguments
        args = [
            "cargo", "run", "--",
            "-d", INPUT_FILE_DIR_PATH,
            "-o", output_dir_path, 
            "-p", pipeline_name,
        ]

        # append additional arguments
        if len(additional_agrs) > 0:
            args.extend(additional_agrs)
        
        arg_compute_instances.append(args)

    return arg_compute_instances


# Global flag to indicate whether to stop reading from stdout
STOP_READING = False

def read_stdout(pipe):
    global STOP_READING
    if pipe.stdout is not None:
        for line in iter(pipe.stdout.readline, ''):
            if STOP_READING:
                break
            print(line.strip())

# -------------------- run the executables -------------------- #
def run_executables(cli: CLIArguments, arg_compute_instances: list[list[str]]) -> None:
    """
    Run the executables.
    """

    # CLI: run only the selected compute instance   
    if cli.args.run_selected is not None:
        # check if the selected compute instances exist
        for selected in cli.args.run_selected:
            if selected < 0 or selected >= len(arg_compute_instances):
                print(f"🔴 Selected compute instance {cli.args.run_selected} does not exist.")
                exit()

        # run only the selected compute instance
        selected_arg_compute_instances = []
        for selected in cli.args.run_selected:
            selected_compt_inst = arg_compute_instances[selected]
            selected_arg_compute_instances.append(selected_compt_inst)
            print(f"🔷 Selected compute instance: {selected_compt_inst}")
    
        arg_compute_instances = selected_arg_compute_instances

    if DRY_RUN:
        print("🔶 Dry run, not running the executables.")
        exit()
    else:
        print("Running the executables...")

        # run the executables
        for args in tqdm.tqdm(arg_compute_instances):  # Replace arg_compute_instances with your own list
            with subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=1, universal_newlines=True) as popen:
                if popen.stdout is not None:
                    thread = threading.Thread(target=read_stdout, args=(popen,))
                    thread.start()

                    try:
                        if COMPUTE_INSTANCE_TIMEOUT > 0:
                            thread.join(timeout=COMPUTE_INSTANCE_TIMEOUT)
                            if thread.is_alive():
                                print(f"🟣 Timeout reached for compute instance: {args}")
                                STOP_READING = True  # Set the flag to stop the thread
                                popen.terminate()
                                thread.join()  # Make sure to join the thread even after termination
                                STOP_READING = False  # Reset the flag
                        else:
                            thread.join()
                    except Exception as e:
                        print(f"🔴 An error occurred: {e}")
                        STOP_READING = True  # Set the flag to stop the thread
                        popen.terminate()
                        thread.join()
                        STOP_READING = False  # Reset the flag
                else:
                    print(f"🔴 Failed compute instance: {args}")
                    sys.exit(1)


if __name__ == "__main__":
    start = datetime.now()

    cli = CLIArguments()

    arg_compute_instances = build_arg_compute_instances(cli)

    # print the commands with their arguments
    print(f"Number of compute instances: {len(arg_compute_instances)}")
    for i in range(len(arg_compute_instances)):
        arg_str = " ".join(arg_compute_instances[i])
        print(f" + [Compute instance: {i}] {arg_str}")

    run_executables(cli, arg_compute_instances)

    end = datetime.now()
    duration = end - start
    human_readable_duration = "hours: {0}, minutes: {1}, seconds: {2}".format(
        duration.seconds // 3600,
        (duration.seconds // 60) % 60,
        duration.seconds % 60
    )
    print(f"🏁 Finished! Total time: {human_readable_duration}")
