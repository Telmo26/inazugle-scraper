import requests
from bs4 import BeautifulSoup

BASE_URL = "https://zukan.inazuma.jp"   # Replace with real domain


# ------------------------------
# Parse the character list page
# ------------------------------
def parse_character_list(html):
    soup = BeautifulSoup(html, "html.parser")

    container = soup.find("div", class_="charaListResult")
    if not container:
        return []

    table = container.find("table")
    if not table:
        return []

    results = []

    for tbody in table.find_all("tbody"):
        rows = tbody.find_all("tr")
        if len(rows) < 2:
            continue

        checkbox = tbody.find("input", class_="my-team-checkbox")
        if not checkbox:
            continue

        char_id = checkbox["data-chara-id"]
        name = checkbox["data-chara-name"]
        nickname = checkbox["data-nickname"]

        # Character page link
        link_tag = tbody.select_one(".nameBox a[href*='chara_param']")
        page_link = link_tag["href"] if link_tag else ""

        results.append({
            "id": char_id,
            "name": name,
            "nickname": nickname,
            "page_link": BASE_URL + page_link
        })

    return results


# ------------------------------
# Parse a character detail page
# ------------------------------
def parse_character_page(html):
    soup = BeautifulSoup(html, "html.parser")

    detail = soup.find("div", class_="detailBox")
    if not detail:
        return {}

    out = {}

    # ------------------- Nickname -------------------
    nickname_tag = detail.select_one(".lBox .nickname ruby, .lBox .nickname")
    out["nickname"] = nickname_tag.get_text(strip=True) if nickname_tag else ""

    # ------------------- Image -------------------
    img_tag = detail.select_one(".lBox img")
    out["image"] = img_tag["src"] if img_tag else ""

    # ------------------- Game -------------------
    game_tag = detail.select_one("dl.appearedWorks dd")
    out["game"] = game_tag.get_text(strip=True) if game_tag else ""

    # ------------------- Description -------------------
    desc_tag = detail.select_one("p.description")
    out["description"] = desc_tag.get_text(" ", strip=True) if desc_tag else ""

    # ------------------- How to Obtain -------------------
    obtain_section = detail.select_one("dl.getTxt")
    out["how_to_obtain"] = obtain_section.get_text(" ", strip=True) if obtain_section else ""

    # ------------------- Stats -------------------
    out["stats"] = {}
    stat_blocks = detail.select("ul.param > li")

    for block in stat_blocks:
        stat_name_tag = block.find("dt")
        value_tag = block.find("td")

        if stat_name_tag and value_tag:
            stat_name = stat_name_tag.get_text(strip=True)
            value = value_tag.get_text(strip=True)
            out["stats"][stat_name] = value

        # Element & Position are special pair
        pos = block.find("dl")
        element = block.find("dl", class_="box")
        if pos and pos.find("dt") and pos.find("dt").get_text(strip=True) == "Position":
            out["position"] = pos.find("dd").get_text(strip=True)
        if element:
            out["element"] = element.find("dd").get_text(strip=True)

    # ------------------- Basic Info -------------------
    out["basic"] = {}
    for li in detail.select("ul.basic li"):
        dt = li.find("dt")
        dd = li.find("dd")
        if dt and dd:
            out["basic"][dt.get_text(strip=True)] = dd.get_text(strip=True)

    return out


# ------------------------------
# Main workflow
# ------------------------------
def scrape_all_characters(main_html):
    characters = parse_character_list(main_html)

    full_data = []

    for c in characters:
        print(f"Fetching: {c['name']} → {c['page_link']}")
        r = requests.get(c["page_link"])
        detail = parse_character_page(r.text)

        full_entry = {**c, **detail}
        full_data.append(full_entry)

    return full_data


# ------------------------------
# Example usage
# ------------------------------
if __name__ == "__main__":
    with open("response.txt", "r", encoding="utf-8") as f:
        html = f.read()

    results = scrape_all_characters(html)
    for r in results:
        print(r)
        print("------------")
