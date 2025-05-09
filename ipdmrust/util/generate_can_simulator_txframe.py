import sys

def process_can_frames(file_path, target_ids):
    with open(file_path, 'r') as file:
        lines = file.readlines()
    
    # Keep the original order from command line
    target_ids_list = target_ids.split(',')
    target_ids_int = [int(id, 16) for id in target_ids_list]
    
    found_ids = {}
    for line in lines:
        parts = line.split()
        if len(parts) >= 3:
            current_id = int(parts[2], 16)
            if current_id in target_ids_int and current_id not in found_ids:
                data_length = int(parts[3].strip('[]'))
                data_bytes = parts[-data_length:]
                hex_bytes = [bytes.fromhex(byte) for byte in data_bytes]
                combined_bytes = b''.join(hex_bytes)
                hex_format = ''.join([f'\\x{b:02x}' for b in combined_bytes])
                
                found_ids[current_id] = hex_format

    for i, can_id in enumerate(target_ids_int):
        if can_id in found_ids:  # Check if the ID was found in the file
            formatted_line = f"""        if self.i % {len(target_ids_int)} == {i} {{
            self.txbuf.push(bxcan::Frame::new_data(
                bxcan::StandardId::new(0x{can_id:X}).unwrap(),
                bxcan::Data::new(b"{found_ids[can_id]}").unwrap(),
            ));
        }}"""
            sys.stdout.write(formatted_line + '\n')
        else:
            print(f"ID not found: 0x{can_id:X}", file=sys.stderr)

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python script.py  ")
        print("Example: python script.py can_data.txt 0x1da,0x285")
        sys.exit(1)
    
    file_path = sys.argv[1]
    target_ids = sys.argv[2]
    
    process_can_frames(file_path, target_ids)
