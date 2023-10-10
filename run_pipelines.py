import os
import subprocess
import tqdm

INPUT_FILE_DIR_PATH = "/home/onyr/code/phdtrack/phdtrack_data_clean"

PIPELINES_NAMES_TO_ADDITIONAL_ARGS: dict[str, list[str]] = {
    "value-node-embedding": [],
    "chunk-top-vn-semantic-embedding": [],
    "graph": [],
    "graph": ["-v", "-a", "chunk-header-node"],
    "chunk-semantic-embedding": ["-v", "-a", "chunk-header-node"],
    "chunk-statistic-embedding": ["-a", "chunk-header-node"], 
}

LIST_ENTROPY_FILTERING_FLAGS = [
    "none",
    "only-max-entropy",
    "min-of-chunk-treshold-entropy",
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
        # add file path or directory path argument
        parser.add_argument(
            '--input',
            type=str,
            help="Input as file path or directory path"
        )
        # select only a single compute instance
        parser.add_argument(
            '--run-selected',
            type=int,
            help="Run selected compute instance only"
        )

        # save parsed arguments
        self.args = parser.parse_args()

        # overwrite debug flag
        global DRY_RUN
        DRY_RUN = True if self.args.dry_run else False

        # overwrite input dir path
        global INPUT_FILE_DIR_PATH
        if self.args.input is not None:
            INPUT_FILE_DIR_PATH = self.args.input
        
        # check input dir path
        if not os.path.exists(INPUT_FILE_DIR_PATH):
            print(f"🔴 Input path {INPUT_FILE_DIR_PATH} does not exist. Abort processing.")
            exit()

        # log parsed arguments
        print("Parsed program params:")
        for arg in vars(self.args):
            print("{0}: {1}".format(
                arg, getattr(self.args, arg)
            ))

def build_arg_compute_instances():
    """
    Create a list of CLI commands with arguments to run the executables.
    """

    # create a list of arguments to run the executables
    arg_compute_instances: list[list[str]] = []

    for pipeline_name in PIPELINES_NAMES_TO_ADDITIONAL_ARGS.keys():
        for entropy_filtering_flag in LIST_ENTROPY_FILTERING_FLAGS:      
            # output dir preparation
            current_dir = os.getcwd()
            output_dir_path = current_dir + "/data/" + pipeline_name.replace("-", "_") + "_-e_" + entropy_filtering_flag
            
            # create output dir if it does not exist
            if not os.path.exists(output_dir_path):
                os.makedirs(output_dir_path)
            else:
                # remove all ".csv" files in the output dir
                for filename in os.listdir(output_dir_path):
                    if filename.endswith(".csv"):
                        os.remove(os.path.join(output_dir_path, filename))
                        print(f" 󰆴 -> Removed {filename} in {output_dir_path}")
            
            # prepare arguments
            args = [
                "cargo", "run", "--",
                "-d", INPUT_FILE_DIR_PATH,
                "-o", output_dir_path, 
                "-p", pipeline_name,
                "-e", entropy_filtering_flag,
            ]

            # append additional arguments
            if len(PIPELINES_NAMES_TO_ADDITIONAL_ARGS[pipeline_name]) > 0:
                args.extend(PIPELINES_NAMES_TO_ADDITIONAL_ARGS[pipeline_name])

            arg_compute_instances.append(args)
        
    return arg_compute_instances


# -------------------- run the executables -------------------- #
def run_executables(cli: CLIArguments, arg_compute_instances: list[list[str]]) -> None:
    """
    Run the executables.
    """

    # CLI: run only the selected compute instance   
    if cli.args.run_selected is not None:
        # check if the selected compute instance exists
        if cli.args.run_selected < 0 or cli.args.run_selected >= len(arg_compute_instances):
            print(f"🔴 Selected compute instance {cli.args.run_selected} does not exist.")
            exit()

        # run only the selected compute instance
        selected_arg_compute_instances = arg_compute_instances[cli.args.run_selected]
        print(f"🔷 Selected compute instance: {selected_arg_compute_instances}")
        arg_compute_instances = [selected_arg_compute_instances]

    if DRY_RUN:
        print("🔶 Dry run, not running the executables.")
        exit()
    else:
        print("Running the executables...")

        # run the executables
        from asyncio import sleep
        import time
        import tqdm

        # get time
        start_time = time.time()

        # run compute instances
        for args in tqdm.tqdm(arg_compute_instances):
            with subprocess.Popen(args, stdout=subprocess.PIPE) as popen:
                for line in iter(popen.stdout.readline, b''):
                    print(line.decode().strip())
                popen.wait()

                args_str = " ".join(args)
                print(f"🟢 Finished compute instance: {args_str}")

        # end time
        end_time = time.time()

        # print time in hours and minutes and seconds
        print("Total time: {} hours, {} minutes, {} seconds".format(
            int((end_time - start_time) // 3600),
            int((end_time - start_time) // 60),
            int((end_time - start_time) % 60)
        ))

if __name__ == "__main__":
    cli = CLIArguments()

    arg_compute_instances = build_arg_compute_instances()

    # print the commands with their arguments
    print(f"Number of compute instances: {len(arg_compute_instances)}")
    for i in range(len(arg_compute_instances)):
        arg_str = " ".join(arg_compute_instances[i])
        print(f" + [Compute instance: {i}] {arg_str}")

    run_executables(cli, arg_compute_instances)
