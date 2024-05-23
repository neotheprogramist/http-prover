import json

def combine_json_files(file1, file2, output_file):
    with open(file1, 'r') as f:
        data1 = json.load(f)
    
    with open(file2, 'r') as f:
        data2 = json.load(f)
    
    combined_data = {
        "program": data1,
        "program_input": data2
    }
    
    with open(output_file, 'w') as f:
        json.dump(combined_data, f, indent=4)

if __name__ == "__main__":
    import sys
    combine_json_files(sys.argv[1], sys.argv[2], sys.argv[3])
