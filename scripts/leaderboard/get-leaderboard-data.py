import requests
import json

def fetch_data(url):
    response = requests.get(url)
    if response.status_code == 200:
        return response.json()
    else:
        print("Failed to fetch data from the API.")
        return None

def process_data(data):
    addresses = []
    points = []
    if data and 'data' in data:
        for item in data['data']:
            if 'address' in item and 'points' in item:
                addresses.append(item['address'])
                points.append(item['points'])
    return addresses, points

def write_to_json(addresses, points, output_file):
    data = [{'address': address, 'points': point} for address, point in zip(addresses, points)]
    with open(output_file, 'w') as f:
        json.dump(data, f, indent=4)

def main():
    url = "https://leaderboard-backend.tangle.tools/leaderboard?skip=0&limit=2000" # we only have ~1200
    output_file = "leaderboard_data.json"

    # Fetch data from the API
    data = fetch_data(url)
    if data:
        # Process the fetched data
        addresses, points = process_data(data)
        
        # Write data to JSON file
        write_to_json(addresses, points, output_file)
        print("Data has been written to", output_file)
    else:
        print("Exiting...")

if __name__ == "__main__":
    main()
