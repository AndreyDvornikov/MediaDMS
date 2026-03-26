import psycopg2
import random
import os
import glob

# Настройки подключения
DB_CONFIG = {
    "dbname": "media_dms_db",
    "user": "admin",
    "password": "access",
    "host": "localhost",
    "port": "5531"
}

def get_image_list(pattern):
    """Находит все файлы, подходящие под паттерн (например, 'author_pic*.*')"""
    # Ищем и png, и jpg, и jpeg
    files = []
    for ext in ['*.png', '*.jpg', '*.jpeg']:
        files.extend(glob.glob(os.path.join(os.path.dirname(__file__), pattern + ext)))
    return files

def read_image_to_bytes(file_path):
    """Читает файл и возвращает байты для BYTEA"""
    if file_path and os.path.exists(file_path):
        with open(file_path, "rb") as f:
            return f.read()
    return None

def seed_database():
    conn = None
    try:
        conn = psycopg2.connect(**DB_CONFIG)
        cur = conn.cursor()
        print("--- Соединение с БД установлено ---")

        # 0. Очистка таблиц перед сидом (чтобы данные не дублировались)
        # TRUNCATE удаляет всё и сбрасывает счетчики ID (RESTART IDENTITY)
        print("Очистка старых данных...")
        cur.execute("TRUNCATE TABLE authors, author_images, albums, songs RESTART IDENTITY CASCADE;")

        # Собираем списки доступных файлов
        author_files = get_image_list("author_pic*")
        album_files = get_image_list("album_pic*")
        
        print(f"Найдено картинок авторов: {len(author_files)}")
        print(f"Найдено картинок альбомов: {len(album_files)}")

        # Тестовые наборы данных
        author_names = ["Siberian Flames", "Deep Code", "Null Pointer", "Postgres Rebels", "The Rustaceans"]
        album_prefixes = ["Memories of", "Echoes from", "The Legacy of", "Inside the"]
        song_titles = ["Basements of Old Town", "Memory Leak", "Deadlock Waltz", "Query King", "Byte Dance", "Invisible Thread"]

        for i in range(len(author_names)):
            # 1. Создаем Автора
            name = author_names[i]
            desc = f"Legendary project from Novosibirsk. Formed in 2026. Major influence on software metal."
            
            cur.execute(
                "INSERT INTO authors (name, description) VALUES (%s, %s) RETURNING author_id;",
                (name, desc)
            )
            author_id = cur.fetchone()[0]
            print(f"Добавлен автор: {name} (ID: {author_id})")

            # 2. Добавляем картинки автора (случайные из найденных в папке)
            if author_files:
                num_to_add = min(len(author_files), random.randint(2, 3))
                chosen_pics = random.sample(author_files, num_to_add)
                for pic_path in chosen_pics:
                    img_data = read_image_to_bytes(pic_path)
                    cur.execute(
                        "INSERT INTO author_images (author_id, image_data) VALUES (%s, %s);",
                        (author_id, psycopg2.Binary(img_data))
                    )

            # 3. Добавляем Альбомы (2-4 штуки)
            for _ in range(random.randint(2, 4)):
                album_name = f"{random.choice(album_prefixes)} {name}"
                year = random.randint(1990, 2026)
                album_desc = f"Recorded in a cold basement during a coding marathon."
                
                # Обложка альбома (случайная одна из папки)
                cover_path = random.choice(album_files) if album_files else None
                cover_data = read_image_to_bytes(cover_path)
                
                cur.execute(
                    """INSERT INTO albums (author_id, name, year, description, cover_data) 
                       VALUES (%s, %s, %s, %s, %s) RETURNING album_id;""",
                    (author_id, album_name, year, album_desc, psycopg2.Binary(cover_data) if cover_data else None)
                )
                album_id = cur.fetchone()[0]

                # 4. Добавляем Песни (2-5 штук)
                for _ in range(random.randint(2, 5)):
                    s_name = random.choice(song_titles)
                    duration = random.randint(120, 400)
                    api_link = f"https://api.music-source.com/v1/stream/{random.getrandbits(32)}"
                    
                    cur.execute(
                        """INSERT INTO songs (album_id, name, duration, link_to_api) 
                           VALUES (%s, %s, %s, %s);""",
                        (album_id, s_name, duration, api_link)
                    )
            
            print(f"  -> Наполнено альбомами и песнями для {name}")

        conn.commit()
        print("\n--- Все данные успешно загружены! ---")

    except Exception as e:
        print(f"\n[!] Ошибка при сиде: {e}")
        if conn:
            conn.rollback()
    finally:
        if conn:
            conn.close()

if __name__ == "__main__":
    seed_database()