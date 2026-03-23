-- 1. Таблица авторов (Главный родитель)
CREATE TABLE IF NOT EXISTS authors (
    author_id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT
);

-- 2. Таблица изображений автора (Связь 1:N)
CREATE TABLE IF NOT EXISTS author_images (
    image_id SERIAL PRIMARY KEY,
    author_id INT NOT NULL,
    image_data BYTEA NOT NULL,
    CONSTRAINT fk_author_image 
        FOREIGN KEY (author_id) 
        REFERENCES authors(author_id) 
        ON DELETE CASCADE
);

-- 3. Таблица альбомов (Дочь автора)
CREATE TABLE IF NOT EXISTS albums (
    album_id SERIAL PRIMARY KEY,
    author_id INT NOT NULL,
    name VARCHAR(255) NOT NULL,
    year INT CHECK (year > 0 AND year < 2100),
    description TEXT,
    cover_data BYTEA,
    CONSTRAINT fk_author_album 
        FOREIGN KEY (author_id) 
        REFERENCES authors(author_id) 
        ON DELETE CASCADE
);

-- 4. Таблица песен (Внучка автора / Дочь альбома)
CREATE TABLE IF NOT EXISTS songs (
    song_id SERIAL PRIMARY KEY,
    album_id INT NOT NULL,
    author_id INT NOT NULL, -- Денормализация для ускорения поиска по автору
    name VARCHAR(255) NOT NULL,
    duration INT NOT NULL CHECK (duration > 0),
    link_to_api TEXT,
    CONSTRAINT fk_album_song 
        FOREIGN KEY (album_id) 
        REFERENCES albums(album_id) 
        ON DELETE CASCADE,
    CONSTRAINT fk_author_song 
        FOREIGN KEY (author_id) 
        REFERENCES authors(author_id) 
        ON DELETE CASCADE
);

-- 5. Индексы для оптимизации поиска (Чтобы Андрюхин Rust летал)
CREATE INDEX IF NOT EXISTS idx_authors_name ON authors(name);
CREATE INDEX IF NOT EXISTS idx_albums_author ON albums(author_id);
CREATE INDEX IF NOT EXISTS idx_songs_album ON songs(album_id);
CREATE INDEX IF NOT EXISTS idx_songs_author ON songs(author_id);
CREATE INDEX IF NOT EXISTS idx_songs_name ON songs(name);