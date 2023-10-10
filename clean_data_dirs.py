import os
import shutil

def remove_all_dirs_in_path(path):
    for filename in os.listdir(path):
        file_path = os.path.join(path, filename)
        if os.path.isdir(file_path):
            shutil.rmtree(file_path)
            print(f"Removed directory: {file_path}")

if __name__ == "__main__":
    current_dir = os.getcwd()
    output_dir_path = os.path.join(current_dir, "data")
    
    if os.path.exists(output_dir_path):
        remove_all_dirs_in_path(output_dir_path)
    else:
        print(f"The directory {output_dir_path} doesn't exist.")
