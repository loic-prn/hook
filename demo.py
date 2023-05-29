import requests
import json
import os
from datetime import datetime
import argparse
import threading

argParser = argparse.ArgumentParser()
argParser.add_argument("-t", "--target", help="the targer account")
argParser.add_argument("-s", "--sessionid", help="the sessionid")

list_of_urls = []
lock = threading.Lock()


def main():
    BASE_URL = "https://www.instagram.com/"
    END = "/?__a=1&__d=dis"
    target = argParser.parse_args().target
    sessionid = argParser.parse_args().sessionid
    headers = {"cookie": "sessionid={sessionid}".format(sessionid=sessionid)}
    session = requests.Session()
    r = session.get(BASE_URL + target + END, headers=headers)
    cookie = session.cookies.get_dict()
    cookie["sessionid"] = sessionid
    session.cookies.update(cookie)
    print("first request")
    r = session.get(BASE_URL + target + END)
    data = json.loads(r.text)
    id = data["graphql"]["user"]["id"]
    print("Getting profile picture")
    download_profile_picture(data)
    print("Loading all the data")
    edges = load_all(session, id)
    print("getting images url")
    thread_list = []
    for edge in edges:
        shortcode = edge["node"]["shortcode"] 
        if threading.active_count() < 5:
            t = threading.Thread(target=get_media_url, args=(session, shortcode))
            t.start()
            thread_list.append(t)
        else:
            for t in thread_list:
                t.join()
            thread_list = []
    
    for t in thread_list:
        t.join()
    
    path = os.path.exists(target)
    if not path:
        os.makedirs(target)
    
    sub_path = target + "/" + datetime.today().strftime('%Y-%m-%d-%H-%M-%S')
    if not os.path.exists(sub_path):
        os.makedirs(sub_path)
    

    print("Downloading everything")
    download_all(list_of_urls, session, target, sub_path)
    print("finished")

def download_profile_picture(data):
    profile_pic_url = data["graphql"]["user"]["profile_pic_url_hd"]
    lock.acquire()
    list_of_urls.append((profile_pic_url, "profile-pic"))
    lock.release()

def download_image(item, session, target, sub_path):
    url = item[0]
    
    if item[1] == "profile-pic":
        prefix = item[1]
    else:
        prefix = item[1].strftime('%Y-%m-%d-%H-%M-%S')

    r = session.get(url)
    extension = ".jpg"
    if ".mp4" in url:
        extension = ".mp4"
    path_to_save = sub_path + "/" + prefix + "-" + target
    i = 1
    while os.path.exists(path_to_save + "-" + str(i) + extension):
        i += 1
    path_to_save = path_to_save + "-" + str(i)

    with open(path_to_save + extension, "wb") as f:
        f.write(r.content)


def download_all(list_of_urls, session, target, sub_path):
        thread_list = []
        for item in list_of_urls:
            if threading.active_count() < 5:
                t = threading.Thread(target=download_image, args=(item, session, target, sub_path))
                t.start()
                thread_list.append(t)
            else:
                for t in thread_list:
                    t.join()
                thread_list = []
        
        for t in thread_list:
            t.join()


def load_all(session, id):
    after = ""
    edges = []
    while True:
        url = "https://www.instagram.com/graphql/query/?query_hash=472f257a40c653c64c666ce877d59d2b&variables={\"id\":\"" + id + "\",\"first\":12,\"after\":\"" + after + "\"}";
        r = session.get(url)
        data = json.loads(r.text)
        edges += data["data"]["user"]["edge_owner_to_timeline_media"]["edges"]
        has_next_page = data["data"]["user"]["edge_owner_to_timeline_media"]["page_info"]["has_next_page"]
        if has_next_page:
            after = data["data"]["user"]["edge_owner_to_timeline_media"]["page_info"]["end_cursor"]
        else:
            break
    
    print("finish loading everything")
    return edges

def get_media_url(session, shortcode):
    url = "https://www.instagram.com/p/" + shortcode + "/?__a=1&__d=dis"
    r = session.get(url)
    data = json.loads(r.text)
    timestamp = datetime.fromtimestamp(data["items"][0]["taken_at"])
    # is_carousel is false if it doesn't exist
    is_carousel = data["items"][0].get("carousel_media", False)
    is_video = data["items"][0].get("video_versions", False)
    if is_carousel:
        for media in data["items"][0]["carousel_media"]:
            candidates = media["image_versions2"]["candidates"]
            max_url = best_url_from_candidates(candidates)
            lock.acquire()
            list_of_urls.append((max_url, timestamp))
            lock.release()

    elif is_video:
        candidates = data["items"][0]["video_versions"]
        max_url = best_url_from_candidates(candidates)
        lock.acquire()
        list_of_urls.append((max_url, timestamp))
        lock.release()

    elif data["items"][0]["image_versions2"]:
        candidates = data["items"][0]["image_versions2"]["candidates"]
        max_url = best_url_from_candidates(candidates)
        lock.acquire()
        list_of_urls.append((max_url, timestamp))
        lock.release()


def best_url_from_candidates(candidates):
    max_pixels = 0
    for candidate in candidates:
        pixels = candidate["width"] * candidate["height"]
        if pixels > max_pixels:
            max_pixels = pixels
            max_url = candidate["url"]
    
    return max_url


if __name__ == "__main__":
    main()