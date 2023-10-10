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

# print the commands with their arguments
print(f"Number of compute instances: {len(arg_compute_instances)}")
for i in range(len(arg_compute_instances)):
    print(f"Compute instance {i}: {arg_compute_instances[i]}")

# run the executables
from asyncio import sleep
import time
import tqdm

# get time
start_time = time.time()

for args in tqdm.tqdm(arg_compute_instances):
    with subprocess.Popen(args, stdout=subprocess.PIPE) as popen:
        for line in iter(popen.stdout.readline, b''):
            print(line.decode().strip())
        popen.wait()

# end time
end_time = time.time()

# print time in hours and minutes and seconds
print("Total time: {} hours, {} minutes, {} seconds".format(
    int((end_time - start_time) // 3600),
    int((end_time - start_time) // 60),
    int((end_time - start_time) % 60)
))