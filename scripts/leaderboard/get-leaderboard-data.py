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
    addresses_and_points = []
    for participant in data['data']['participants']:
        # print("+++++++++++++++++++ \n")
        # print(participant)
        # print(participant['points'])

        for address in participant['addresses']:
            if address['type'] == "stash":
                print(address['address'])
                addresses_and_points.append((address['address'], participant['points']))
    
    return addresses_and_points

def write_to_json(addresses_and_points, output_file):
    #data = [{'address': address, 'points': point} for address, point in zip(addresses, points)]
    # Convert list of tuples to dictionary
    formatted_data = {address: points for address, points in addresses_and_points}
    with open(output_file, 'w') as f:
        json.dump(formatted_data, f, indent=4)

def main():
    url = "https://leaderboard-backend.tangle.tools/leaderboard?skip=0&limit=500" # we only have ~1200, change this once we fix api endpoint
    output_file = "leaderboard_data.json"

    # Fetch data from the API
    data = fetch_data(url)
    if data:
        # Process the fetched data
        addresses_and_points = process_data(data)
        print(addresses_and_points)
        
        # # Write data to JSON file
        write_to_json(addresses_and_points, output_file)
        print("Data has been written to", output_file)
    else:
        print("Exiting...")

if __name__ == "__main__":
    main()
