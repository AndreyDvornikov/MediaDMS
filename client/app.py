import sys
import requests
import tempfile
import pygame
import os

from PyQt6.QtWidgets import *
from PyQt6.QtCore import *
from PyQt6.QtGui import QPixmap

SERVER_URL = "http://127.0.0.1:8080/api/v1/query"

pygame.mixer.init()

PLACEHOLDER_PATH = "covers/placeholder.png"


def parse_range(text):
    if not text:
        return None
    if "-" in text:
        try:
            a, b = map(int, text.split("-"))
            return {"min": a, "max": b}
        except:
            return None
    try:
        v = int(text)
        return {"min": v, "max": v}
    except:
        return None


class App(QWidget):
    def __init__(self):
        super().__init__()

        self.setWindowTitle("MediaDMS Client")
        self.setGeometry(100, 100, 1200, 700)

        self.sort_state = {}

        layout = QVBoxLayout()
        grid = QGridLayout()

        # -------- SEARCH --------
        self.song_id = QLineEdit(); self.song_id.setPlaceholderText("id 10-72")
        self.author = QLineEdit(); self.author.setPlaceholderText("author")
        self.song = QLineEdit(); self.song.setPlaceholderText("song")
        self.album_search = QLineEdit(); self.album_search.setPlaceholderText("album")
        self.year = QLineEdit(); self.year.setPlaceholderText("year")
        self.duration = QLineEdit(); self.duration.setPlaceholderText("duration")

        grid.addWidget(self.song_id, 0, 0)
        grid.addWidget(self.author, 0, 1)
        grid.addWidget(self.song, 0, 2)
        grid.addWidget(self.album_search, 0, 3)
        grid.addWidget(self.year, 0, 4)
        grid.addWidget(self.duration, 0, 5)

        btn_search = QPushButton("Искать")
        btn_search.clicked.connect(self.search)

        btn_reset = QPushButton("Сброс")
        btn_reset.clicked.connect(self.load_all_tracks)

        grid.addWidget(btn_search, 1, 0)
        grid.addWidget(btn_reset, 1, 1)

        layout.addLayout(grid)

        # -------- TABLE --------
        self.table = QTableWidget()
        self.table.setColumnCount(8)
        self.table.setHorizontalHeaderLabels([
            "#", "Название", "Автор", "Альбом", "Год", "Длительность", "ID", "Обложка"
        ])
        self.table.setSelectionBehavior(QTableWidget.SelectionBehavior.SelectRows)

        self.table.horizontalHeader().sectionClicked.connect(self.on_header_click)

        layout.addWidget(self.table)

        self.status = QLabel("Ready")
        layout.addWidget(self.status)

        # КНОПКА СВОРАЧИВАНИЯ
        self.toggle_btn = QPushButton("Показать добавление")
        self.toggle_btn.setCheckable(True)
        self.toggle_btn.clicked.connect(self.toggle_create_panel)
        layout.addWidget(self.toggle_btn)

        # -------- CREATE --------
        self.create_box = QGroupBox("Добавление")
        self.create_box.setVisible(False)

        c_layout = QVBoxLayout()

        self.author_name = QLineEdit(); self.author_name.setPlaceholderText("author name")
        self.bio = QLineEdit(); self.bio.setPlaceholderText("bio")
        btn_add_author = QPushButton("Добавить автора")
        btn_add_author.clicked.connect(self.create_author)

        self.author_id_input = QLineEdit(); self.author_id_input.setPlaceholderText("author_id")
        self.album_name = QLineEdit(); self.album_name.setPlaceholderText("album")
        self.album_year = QLineEdit(); self.album_year.setPlaceholderText("year")
        btn_add_album = QPushButton("Добавить альбом")
        btn_add_album.clicked.connect(self.create_album)

        self.album_id_input = QLineEdit(); self.album_id_input.setPlaceholderText("album_id")
        self.song_name_input = QLineEdit(); self.song_name_input.setPlaceholderText("song name")
        self.duration_input = QLineEdit(); self.duration_input.setPlaceholderText("duration")
        btn_add_song = QPushButton("Добавить песню")
        btn_add_song.clicked.connect(self.create_song)

        c_layout.addWidget(QLabel("Автор"))
        c_layout.addWidget(self.author_name)
        c_layout.addWidget(self.bio)
        c_layout.addWidget(btn_add_author)

        c_layout.addWidget(QLabel("Альбом"))
        c_layout.addWidget(self.author_id_input)
        c_layout.addWidget(self.album_name)
        c_layout.addWidget(self.album_year)
        c_layout.addWidget(btn_add_album)

        c_layout.addWidget(QLabel("Песня"))
        c_layout.addWidget(self.album_id_input)
        c_layout.addWidget(self.song_name_input)
        c_layout.addWidget(self.duration_input)
        c_layout.addWidget(btn_add_song)

        self.create_box.setLayout(c_layout)
        layout.addWidget(self.create_box)

        self.setLayout(layout)
        self.tracks = []

        QTimer.singleShot(100, self.load_all_tracks)

    # -------- TOGGLE --------

    def toggle_create_panel(self):
        if self.toggle_btn.isChecked():
            self.create_box.setVisible(True)
            self.toggle_btn.setText("Скрыть добавление")
        else:
            self.create_box.setVisible(False)
            self.toggle_btn.setText("Показать добавление")

    # -------- IMAGE --------

    def get_pixmap(self, track):
        url = track.get("url")

        if url:
            try:
                r = requests.get(url, timeout=2)
                pixmap = QPixmap()
                pixmap.loadFromData(r.content)
                return pixmap
            except:
                pass

        if os.path.exists(PLACEHOLDER_PATH):
            return QPixmap(PLACEHOLDER_PATH)

        return None

    # -------- LOAD / SEARCH --------

    def load_all_tracks(self):
        self.search_with_sort(None, None)

    def search(self):
        self.search_with_sort(None, None)

    def search_with_sort(self, field, order):
        filters = {}

        if self.song_id.text():
            val = parse_range(self.song_id.text())
            if val:
                filters["song_id"] = val

        if self.author.text():
            filters["author"] = self.author.text()

        if self.song.text():
            filters["song_name"] = self.song.text()

        if self.album_search.text():
            filters["album_name"] = self.album_search.text()

        if self.year.text():
            val = parse_range(self.year.text())
            if val:
                filters["year"] = val

        if self.duration.text():
            try:
                filters["duration_max"] = int(self.duration.text())
            except:
                pass

        r = requests.post(SERVER_URL, json={
            "method": "read",
            "entity": "song",
            "filters": filters,
            "sort": {"field": field, "order": order}
        })

        data = r.json()
        self.tracks = data.get("data", {}).get("items", []) or []
        self.fill_table(self.tracks)

        self.status.setText(f"Найдено: {len(self.tracks)}")

    # -------- SORT --------

    def on_header_click(self, index):
        columns = ["index", "song_name", "author", "album_name", "year", "duration_sec", "song_id"]

        if index >= len(columns):
            return

        field = columns[index]

        order = self.sort_state.get(field, "asc")
        order = "desc" if order == "asc" else "asc"
        self.sort_state[field] = order

        self.search_with_sort(field, order)

    # -------- TABLE --------

    def fill_table(self, tracks):
        self.table.setRowCount(len(tracks))

        for i, t in enumerate(tracks):
            self.table.setRowHeight(i, 70)

            self.table.setItem(i, 0, QTableWidgetItem(str(i + 1)))
            self.table.setItem(i, 1, QTableWidgetItem(str(t.get("song_name"))))
            self.table.setItem(i, 2, QTableWidgetItem(str(t.get("author"))))
            self.table.setItem(i, 3, QTableWidgetItem(str(t.get("album_name"))))
            self.table.setItem(i, 4, QTableWidgetItem(str(t.get("year"))))
            self.table.setItem(i, 5, QTableWidgetItem(str(t.get("duration_sec"))))
            self.table.setItem(i, 6, QTableWidgetItem(str(t.get("song_id"))))

            pixmap = self.get_pixmap(t)
            if pixmap:
                label = QLabel()
                label.setPixmap(pixmap.scaled(60, 60))
                self.table.setCellWidget(i, 7, label)

    # -------- CREATE --------

    def create_author(self):
        r = requests.post(SERVER_URL, json={
            "method": "write",
            "entity": "author",
            "data": {
                "author_name": self.author_name.text(),
                "bio": self.bio.text(),
                "images_binaries": []
            }
        })
        res = r.json()
        self.status.setText(str(res))

        if "created_id" in res:
            self.author_id_input.setText(str(res["created_id"]))

    def create_album(self):
        r = requests.post(SERVER_URL, json={
            "method": "write",
            "entity": "album",
            "data": {
                "author_id": int(self.author_id_input.text()),
                "album_name": self.album_name.text(),
                "year": int(self.album_year.text()),
                "description": "",
                "cover_binary": ""
            }
        })
        res = r.json()
        self.status.setText(str(res))

        if "created_id" in res:
            self.album_id_input.setText(str(res["created_id"]))

    def create_song(self):
        r = requests.post(SERVER_URL, json={
            "method": "write",
            "entity": "song",
            "data": {
                "album_id": int(self.album_id_input.text()),
                "song_name": self.song_name_input.text(),
                "duration_sec": int(self.duration_input.text()),
                "audio_url": ""
            }
        })
        res = r.json()
        self.status.setText(str(res))
        self.load_all_tracks()


if __name__ == "__main__":
    app = QApplication(sys.argv)
    w = App()
    w.show()
    sys.exit(app.exec())