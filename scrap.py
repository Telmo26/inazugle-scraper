import requests
import json

def send_post_request():
    url = "https://zukan.inazuma.jp/en/chara_list/process_form"   # Replace with your target URL
    payload = {
        "rc": "0",
        "attr_filter": "山",
        "pos_filter": "MF",
        "name_search": "",
        "per_page": "50", # Possible values 50, 100, 150, 200 
    }

    try:
        response = requests.post(url, data=payload)
        response.raise_for_status()  # Raise an error for bad status codes

        # Write raw response text to a file
        with open("response.txt", "w", encoding="utf-8") as f:
            f.write(response.text)

        print("Raw response saved to response.txt")

        # If response is JSON, write parsed JSON to a separate file
        try:
            data = response.json()
            with open("response.json", "w", encoding="utf-8") as f:
                json.dump(data, f, indent=4)
            print("Parsed JSON saved to response.json")
        except ValueError:
            print("Response is not valid JSON. Only raw text was saved.")

    except requests.exceptions.RequestException as e:
        print("Request failed:", e)

if __name__ == "__main__":
    send_post_request()
