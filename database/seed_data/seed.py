import glob
import os
import random
from collections import defaultdict

import psycopg2
import requests


def load_env_file():
    env_path = os.path.join(os.path.dirname(__file__), "..", ".env")
    env_path = os.path.abspath(env_path)

    if not os.path.exists(env_path):
        return

    with open(env_path, "r", encoding="utf-8") as env_file:
        for raw_line in env_file:
            line = raw_line.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue

            key, value = line.split("=", 1)
            os.environ.setdefault(key.strip(), value.strip())


load_env_file()


DB_CONFIG = {
    "dbname": os.getenv("POSTGRES_DB", "media_dms_db"),
    "user": os.getenv("POSTGRES_USER", "admin"),
    "password": os.getenv("POSTGRES_PASSWORD", "access"),
    "host": os.getenv("POSTGRES_HOST", "localhost"),
    "port": os.getenv("DB_PORT", "5531"),
}

JAMENDO_API_URL = "https://api.jamendo.com/v3.0/tracks/"
JAMENDO_CLIENT_ID = os.getenv("JAMENDO_CLIENT_ID", "74d18f42")
JAMENDO_TRACK_LIMIT = int(os.getenv("JAMENDO_TRACK_LIMIT", "100"))
AUTHOR_LIMIT = int(os.getenv("SEED_AUTHOR_LIMIT", "5"))


def get_image_list(pattern):
    files = []
    base_dir = os.path.dirname(__file__)
    for ext in ["*.png", "*.jpg", "*.jpeg"]:
        files.extend(glob.glob(os.path.join(base_dir, pattern + ext)))
    return files


def read_image_to_bytes(file_path):
    if file_path and os.path.exists(file_path):
        with open(file_path, "rb") as f:
            return f.read()
    return None


def fetch_jamendo_tracks(client_id, count=100):
    params = {
        "client_id": client_id,
        "format": "json",
        "limit": count,
        "include": "musicinfo",
        "audioformat": "mp32",
    }

    response = requests.get(JAMENDO_API_URL, params=params, timeout=30)
    response.raise_for_status()
    data = response.json()

    if data.get("headers", {}).get("status") != "success":
        message = data.get("headers", {}).get("error_message", "Unknown Jamendo API error")
        raise RuntimeError(f"Jamendo API error: {message}")

    tracks = [
        track
        for track in data.get("results", [])
        if track.get("audiodownload") and track.get("name") and track.get("artist_name")
    ]

    if not tracks:
        raise RuntimeError("Jamendo returned no usable tracks with audiodownload links")

    return tracks


def group_tracks_by_artist(tracks):
    grouped = defaultdict(list)
    for track in tracks:
        grouped[track["artist_name"]].append(track)
    return grouped


def group_tracks_by_album(tracks):
    grouped = defaultdict(list)
    for track in tracks:
        album_name = track.get("album_name") or "Singles"
        grouped[album_name].append(track)
    return grouped


def parse_year(track):
    for field in ("releasedate", "album_releasedate"):
        value = track.get(field)
        if value and len(value) >= 4 and value[:4].isdigit():
            return int(value[:4])
    return random.randint(1990, 2026)


def choose_seed_artists(tracks, limit):
    grouped = group_tracks_by_artist(tracks)
    candidates = [item for item in grouped.items() if len(item[1]) >= 2]
    if len(candidates) < limit:
        candidates = list(grouped.items())

    random.shuffle(candidates)
    return candidates[:limit]


def seed_database():
    conn = None
    try:
        print("--- Загружаем реальные треки из Jamendo ---")
        jamendo_tracks = fetch_jamendo_tracks(JAMENDO_CLIENT_ID, JAMENDO_TRACK_LIMIT)
        selected_artists = choose_seed_artists(jamendo_tracks, AUTHOR_LIMIT)

        conn = psycopg2.connect(**DB_CONFIG)
        cur = conn.cursor()
        print("--- Соединение с БД установлено ---")

        print("Очистка старых данных...")
        cur.execute("TRUNCATE TABLE authors, author_images, albums, songs RESTART IDENTITY CASCADE;")

        author_files = get_image_list("author_pic*")
        album_files = get_image_list("album_pic*")

        print(f"Найдено картинок авторов: {len(author_files)}")
        print(f"Найдено картинок альбомов: {len(album_files)}")
        print(f"Найдено Jamendo-треков: {len(jamendo_tracks)}")
        print(f"Выбрано авторов для сида: {len(selected_artists)}")

        for artist_name, artist_tracks in selected_artists:
            description = (
                f"{artist_name} imported from Jamendo. "
                "Songs include real downloadable links from the public API."
            )

            cur.execute(
                "INSERT INTO authors (name, description) VALUES (%s, %s) RETURNING author_id;",
                (artist_name, description),
            )
            author_id = cur.fetchone()[0]
            print(f"Добавлен автор: {artist_name} (ID: {author_id})")

            if author_files:
                num_to_add = min(len(author_files), random.randint(2, 3))
                for pic_path in random.sample(author_files, num_to_add):
                    img_data = read_image_to_bytes(pic_path)
                    cur.execute(
                        "INSERT INTO author_images (author_id, image_data) VALUES (%s, %s);",
                        (author_id, psycopg2.Binary(img_data)),
                    )

            albums = list(group_tracks_by_album(artist_tracks).items())
            random.shuffle(albums)

            for album_name, album_tracks in albums[: random.randint(1, min(3, len(albums)))]:
                first_track = album_tracks[0]
                year = parse_year(first_track)
                album_desc = (
                    f"Album seeded from Jamendo API for artist {artist_name}. "
                    f"Contains {len(album_tracks)} imported tracks."
                )

                cover_path = random.choice(album_files) if album_files else None
                cover_data = read_image_to_bytes(cover_path)

                cur.execute(
                    """
                    INSERT INTO albums (author_id, name, year, description, cover_data)
                    VALUES (%s, %s, %s, %s, %s)
                    RETURNING album_id;
                    """,
                    (
                        author_id,
                        album_name,
                        year,
                        album_desc,
                        psycopg2.Binary(cover_data) if cover_data else None,
                    ),
                )
                album_id = cur.fetchone()[0]

                random.shuffle(album_tracks)
                songs_to_insert = album_tracks[: random.randint(1, min(5, len(album_tracks)))]

                for track in songs_to_insert:
                    song_name = track["name"]
                    duration = int(track.get("duration") or random.randint(120, 400))
                    api_link = track["audiodownload"]

                    cur.execute(
                        """
                        INSERT INTO songs (album_id, name, duration, link_to_api)
                        VALUES (%s, %s, %s, %s);
                        """,
                        (album_id, song_name, duration, api_link),
                    )

                print(
                    f"  -> Альбом: {album_name} | песен добавлено: {len(songs_to_insert)}"
                )

        conn.commit()
        print("\n--- Все данные успешно загружены c реальными Jamendo-ссылками! ---")

    except Exception as e:
        print(f"\n[!] Ошибка при сиде: {e}")
        if conn:
            conn.rollback()
    finally:
        if conn:
            conn.close()


if __name__ == "__main__":
    seed_database()
