import base64
import os
import sys
import tempfile

import pygame
import requests
from PyQt6.QtCore import QTimer, Qt
from PyQt6.QtGui import QAction, QIntValidator, QPixmap
from PyQt6.QtWidgets import (
    QAbstractItemView,
    QApplication,
    QFileDialog,
    QFormLayout,
    QGridLayout,
    QGroupBox,
    QHBoxLayout,
    QLabel,
    QLineEdit,
    QListWidget,
    QListWidgetItem,
    QMenu,
    QMessageBox,
    QPushButton,
    QScrollArea,
    QTableWidget,
    QTableWidgetItem,
    QTabWidget,
    QVBoxLayout,
    QWidget,
    QStackedWidget,
)

SERVER_URL = "http://127.0.0.1:8080/api/v1/query"
REQUEST_TIMEOUT = 10

pygame.mixer.init()

BASE_DIR = os.path.dirname(__file__)
PLACEHOLDER_PATH = os.path.join(BASE_DIR, "covers", "placeholder.png")


def parse_range(text):
    if not text:
        return None
    if "-" in text:
        try:
            a, b = map(int, text.split("-", 1))
            return {"min": a, "max": b}
        except ValueError:
            return None
    try:
        value = int(text)
        return {"min": value, "max": value}
    except ValueError:
        return None


def decode_pixmap(binary):
    if not binary:
        return None
    try:
        pixmap = QPixmap()
        pixmap.loadFromData(base64.b64decode(binary))
        if not pixmap.isNull():
            return pixmap
    except Exception:
        return None
    return None


def format_duration(seconds):
    try:
        seconds = int(seconds)
    except (TypeError, ValueError):
        return "Unknown"
    minutes, seconds = divmod(seconds, 60)
    return f"{minutes}:{seconds:02d}"


class ImagePreviewRow(QWidget):
    def __init__(self, limit, title, parent=None):
        super().__init__(parent)
        self.limit = limit
        self.title = title
        self.images = []

        layout = QVBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)

        self.caption = QLabel(f"{self.title}: not selected")
        layout.addWidget(self.caption)

        self.preview_layout = QHBoxLayout()
        self.preview_layout.setContentsMargins(0, 0, 0, 0)
        self.preview_layout.setSpacing(8)
        layout.addLayout(self.preview_layout)
        self._render()

    def set_images(self, images):
        self.images = images[: self.limit]
        self._render()

    def clear(self):
        self.images = []
        self._render()

    def _render(self):
        while self.preview_layout.count():
            item = self.preview_layout.takeAt(0)
            widget = item.widget()
            if widget is not None:
                widget.deleteLater()

        if not self.images:
            self.caption.setText(f"{self.title}: not selected")
            placeholder = QLabel("No preview")
            placeholder.setFixedSize(100, 100)
            placeholder.setAlignment(Qt.AlignmentFlag.AlignCenter)
            placeholder.setStyleSheet("border: 1px dashed #b7b7b7; color: #666;")
            self.preview_layout.addWidget(placeholder)
            self.preview_layout.addStretch()
            return

        self.caption.setText(f"{self.title}: {len(self.images)} file(s)")
        for image in self.images:
            label = QLabel()
            label.setFixedSize(100, 100)
            label.setAlignment(Qt.AlignmentFlag.AlignCenter)
            pixmap = decode_pixmap(image)
            if pixmap:
                label.setPixmap(
                    pixmap.scaled(
                        100,
                        100,
                        Qt.AspectRatioMode.KeepAspectRatio,
                        Qt.TransformationMode.SmoothTransformation,
                    )
                )
            else:
                label.setText("Preview error")
                label.setStyleSheet("border: 1px dashed #b7b7b7; color: #666;")
            self.preview_layout.addWidget(label)
        self.preview_layout.addStretch()


class AuthorPage(QWidget):
    def __init__(self, app):
        super().__init__()
        self.app = app

        outer = QVBoxLayout(self)

        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        outer.addWidget(scroll)

        content = QWidget()
        self.layout = QVBoxLayout(content)
        self.layout.setSpacing(16)
        scroll.setWidget(content)

        header = QHBoxLayout()
        self.image_label = QLabel()
        self.image_label.setFixedSize(220, 220)
        self.image_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.image_label.setStyleSheet("border: 1px solid #d0d0d0; background: #fafafa;")
        header.addWidget(self.image_label, 0, Qt.AlignmentFlag.AlignTop)

        text_block = QVBoxLayout()
        self.name_label = QLabel()
        self.name_label.setWordWrap(True)
        self.name_label.setStyleSheet("font-size: 28px; font-weight: 600;")
        text_block.addWidget(self.name_label)

        self.bio_label = QLabel()
        self.bio_label.setWordWrap(True)
        self.bio_label.setStyleSheet("font-size: 14px; color: #444;")
        text_block.addWidget(self.bio_label)
        text_block.addStretch()
        header.addLayout(text_block, 1)
        self.layout.addLayout(header)

        albums_group = QGroupBox("Albums")
        albums_layout = QVBoxLayout(albums_group)
        self.albums_list = QListWidget()
        self.albums_list.itemClicked.connect(self.open_album)
        albums_layout.addWidget(self.albums_list)
        self.layout.addWidget(albums_group)

        tracks_group = QGroupBox("Tracks")
        tracks_layout = QVBoxLayout(tracks_group)
        self.tracks_list = QListWidget()
        self.tracks_list.itemClicked.connect(self.open_track)
        tracks_layout.addWidget(self.tracks_list)
        self.layout.addWidget(tracks_group)

    def set_content(self, author, albums, tracks):
        self.name_label.setText(author.get("author") or "Author")
        description = author.get("description") or "No bio available."
        self.bio_label.setText(description)
        images = author.get("images_binaries") or []
        self.app.set_image_label(self.image_label, images[0] if images else None, 220, 220)

        self.albums_list.clear()
        for album in albums:
            text = f'{album.get("album_name", "Unknown album")} ({album.get("year", "n/a")})'
            item = QListWidgetItem(text)
            item.setData(Qt.ItemDataRole.UserRole, album)
            self.albums_list.addItem(item)

        self.tracks_list.clear()
        for track in tracks:
            text = f'{track.get("song_name", "Unknown track")} • {format_duration(track.get("duration_sec"))}'
            item = QListWidgetItem(text)
            item.setData(Qt.ItemDataRole.UserRole, track)
            self.tracks_list.addItem(item)

    def open_album(self, item):
        album = item.data(Qt.ItemDataRole.UserRole)
        self.app.open_album_page(album_id=album.get("album_id"))

    def open_track(self, item):
        track = item.data(Qt.ItemDataRole.UserRole)
        self.app.open_track_page(track.get("song_id"))


class AlbumPage(QWidget):
    def __init__(self, app):
        super().__init__()
        self.app = app

        outer = QVBoxLayout(self)

        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        outer.addWidget(scroll)

        content = QWidget()
        self.layout = QVBoxLayout(content)
        self.layout.setSpacing(16)
        scroll.setWidget(content)

        header = QHBoxLayout()
        self.cover_label = QLabel()
        self.cover_label.setFixedSize(240, 240)
        self.cover_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.cover_label.setStyleSheet("border: 1px solid #d0d0d0; background: #fafafa;")
        header.addWidget(self.cover_label, 0, Qt.AlignmentFlag.AlignTop)

        info = QVBoxLayout()
        self.name_label = QLabel()
        self.name_label.setStyleSheet("font-size: 26px; font-weight: 600;")
        self.name_label.setWordWrap(True)
        info.addWidget(self.name_label)

        self.year_label = QLabel()
        info.addWidget(self.year_label)

        self.author_button = QPushButton()
        self.author_button.setFlat(True)
        self.author_button.setCursor(Qt.CursorShape.PointingHandCursor)
        self.author_button.setStyleSheet("text-align: left; color: #2563eb;")
        self.author_button.clicked.connect(self.open_author)
        info.addWidget(self.author_button)

        self.description_label = QLabel()
        self.description_label.setWordWrap(True)
        self.description_label.setStyleSheet("font-size: 14px; color: #444;")
        info.addWidget(self.description_label)
        info.addStretch()

        header.addLayout(info, 1)
        self.layout.addLayout(header)

        tracks_group = QGroupBox("Tracks")
        tracks_layout = QVBoxLayout(tracks_group)
        self.tracks_list = QListWidget()
        self.tracks_list.itemClicked.connect(self.open_track)
        tracks_layout.addWidget(self.tracks_list)
        self.layout.addWidget(tracks_group)

        self.current_album = None

    def set_content(self, album, tracks):
        self.current_album = album
        self.name_label.setText(album.get("album_name") or "Album")
        self.year_label.setText(f'Year: {album.get("year", "Unknown")}')
        self.author_button.setText(f'Author: {album.get("author", "Unknown")}')
        description = album.get("description") or "No description available."
        self.description_label.setText(description)
        self.app.set_image_label(self.cover_label, album.get("cover_binary"), 240, 240)

        self.tracks_list.clear()
        for track in tracks:
            text = f'{track.get("song_name", "Unknown track")} • {format_duration(track.get("duration_sec"))}'
            item = QListWidgetItem(text)
            item.setData(Qt.ItemDataRole.UserRole, track)
            self.tracks_list.addItem(item)

    def open_author(self):
        if self.current_album:
            self.app.open_author_page(self.current_album.get("author"))

    def open_track(self, item):
        track = item.data(Qt.ItemDataRole.UserRole)
        self.app.open_track_page(track.get("song_id"))


class TrackPage(QWidget):
    def __init__(self, app):
        super().__init__()
        self.app = app
        self.current_track = None
        self.current_album = None
        self.current_author = None

        outer = QVBoxLayout(self)

        scroll = QScrollArea()
        scroll.setWidgetResizable(True)
        outer.addWidget(scroll)

        content = QWidget()
        self.layout = QVBoxLayout(content)
        self.layout.setSpacing(16)
        scroll.setWidget(content)

        top = QHBoxLayout()

        self.cover_label = QLabel()
        self.cover_label.setFixedSize(220, 220)
        self.cover_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.cover_label.setStyleSheet("border: 1px solid #d0d0d0; background: #fafafa;")
        top.addWidget(self.cover_label, 0, Qt.AlignmentFlag.AlignTop)

        right = QVBoxLayout()
        self.name_label = QLabel()
        self.name_label.setWordWrap(True)
        self.name_label.setStyleSheet("font-size: 28px; font-weight: 600;")
        right.addWidget(self.name_label)

        self.duration_label = QLabel()
        right.addWidget(self.duration_label)

        self.author_button = QPushButton()
        self.author_button.setFlat(True)
        self.author_button.setCursor(Qt.CursorShape.PointingHandCursor)
        self.author_button.setStyleSheet("text-align: left; color: #2563eb;")
        self.author_button.clicked.connect(self.open_author)
        right.addWidget(self.author_button)

        self.album_button = QPushButton()
        self.album_button.setFlat(True)
        self.album_button.setCursor(Qt.CursorShape.PointingHandCursor)
        self.album_button.setStyleSheet("text-align: left; color: #2563eb;")
        self.album_button.clicked.connect(self.open_album)
        right.addWidget(self.album_button)

        self.play_button = QPushButton("Play")
        self.play_button.clicked.connect(self.play_track)
        right.addWidget(self.play_button, 0, Qt.AlignmentFlag.AlignLeft)

        controls = QHBoxLayout()
        controls.setSpacing(8)

        self.pause_button = QPushButton("Pause")
        self.pause_button.clicked.connect(self.toggle_pause)
        controls.addWidget(self.pause_button)

        self.stop_button = QPushButton("Stop")
        self.stop_button.clicked.connect(self.stop_track)
        controls.addWidget(self.stop_button)

        controls.addStretch()
        right.addLayout(controls)

        right.addStretch()
        top.addLayout(right, 1)

        self.author_image_label = QLabel()
        self.author_image_label.setFixedSize(160, 160)
        self.author_image_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.author_image_label.setStyleSheet("border: 1px solid #d0d0d0; background: #fafafa;")
        top.addWidget(self.author_image_label, 0, Qt.AlignmentFlag.AlignTop)

        self.layout.addLayout(top)

    def set_content(self, track, album, author):
        self.current_track = track
        self.current_album = album
        self.current_author = author

        self.name_label.setText(track.get("song_name") or "Track")
        self.duration_label.setText(f'Duration: {format_duration(track.get("duration_sec"))}')
        self.author_button.setText(f'Author: {track.get("author", "Unknown")}')
        self.album_button.setText(f'Album: {track.get("album_name", "Unknown")}')
        self.play_button.setEnabled(bool(track.get("audio_url")))
        self.refresh_playback_state()

        self.app.set_image_label(self.cover_label, album.get("cover_binary") if album else None, 220, 220)
        author_image = None
        if author:
            images = author.get("images_binaries") or []
            author_image = images[0] if images else None
        self.app.set_image_label(self.author_image_label, author_image, 160, 160)

    def open_author(self):
        if self.current_track:
            self.app.open_author_page(self.current_track.get("author"))

    def open_album(self):
        if self.current_album:
            self.app.open_album_page(album_id=self.current_album.get("album_id"))

    def play_track(self):
        if self.current_track:
            self.app.play_audio(self.current_track)

    def toggle_pause(self):
        self.app.toggle_pause_playback()

    def stop_track(self):
        self.app.stop_playback()

    def refresh_playback_state(self):
        has_audio = bool(self.current_track and self.current_track.get("audio_url"))
        is_current = self.app.is_current_track(self.current_track)
        self.play_button.setEnabled(has_audio)
        self.pause_button.setEnabled(has_audio and is_current)
        self.stop_button.setEnabled(is_current)
        self.pause_button.setText("Resume" if self.app.playback_paused and is_current else "Pause")


class App(QWidget):
    def __init__(self):
        super().__init__()

        self.setWindowTitle("MediaDMS Client")
        self.setGeometry(100, 100, 1200, 760)

        self.sort_state = {}
        self.tracks = []
        self.navigation_history = []
        self.temp_audio_file = None
        self.current_playing_track = None
        self.playback_paused = False
        self.author_images_binaries = []
        self.album_cover_binary = ""

        self.main_layout = QVBoxLayout(self)

        nav_bar = QHBoxLayout()
        self.back_button = QPushButton("Back")
        self.back_button.clicked.connect(self.go_back)
        self.back_button.setVisible(False)
        nav_bar.addWidget(self.back_button, 0, Qt.AlignmentFlag.AlignLeft)

        self.page_title = QLabel("Tracks")
        self.page_title.setStyleSheet("font-size: 20px; font-weight: 600;")
        nav_bar.addWidget(self.page_title)
        nav_bar.addStretch()
        self.main_layout.addLayout(nav_bar)

        self.stack = QStackedWidget()
        self.main_layout.addWidget(self.stack)

        self.search_page = QWidget()
        self.stack.addWidget(self.search_page)
        self._build_search_page()

        self.author_page = AuthorPage(self)
        self.stack.addWidget(self.author_page)

        self.album_page = AlbumPage(self)
        self.stack.addWidget(self.album_page)

        self.track_page = TrackPage(self)
        self.stack.addWidget(self.track_page)

        self.stack.setCurrentWidget(self.search_page)
        self._update_navigation_ui()
        self.update_form_state()

        QTimer.singleShot(100, self.load_all_tracks)

    def _build_search_page(self):
        layout = QVBoxLayout(self.search_page)
        grid = QGridLayout()

        self.song_id = QLineEdit()
        self.song_id.setPlaceholderText("id 10-72")
        self.author = QLineEdit()
        self.author.setPlaceholderText("author")
        self.song = QLineEdit()
        self.song.setPlaceholderText("song")
        self.album_search = QLineEdit()
        self.album_search.setPlaceholderText("album")
        self.year = QLineEdit()
        self.year.setPlaceholderText("year")
        self.duration = QLineEdit()
        self.duration.setPlaceholderText("duration")

        grid.addWidget(self.song_id, 0, 0)
        grid.addWidget(self.author, 0, 1)
        grid.addWidget(self.song, 0, 2)
        grid.addWidget(self.album_search, 0, 3)
        grid.addWidget(self.year, 0, 4)
        grid.addWidget(self.duration, 0, 5)

        btn_search = QPushButton("Искать")
        btn_search.clicked.connect(self.search)
        grid.addWidget(btn_search, 1, 0)

        btn_reset = QPushButton("Сброс")
        btn_reset.clicked.connect(self.load_all_tracks)
        grid.addWidget(btn_reset, 1, 1)

        layout.addLayout(grid)

        self.table = QTableWidget()
        self.table.setColumnCount(8)
        self.table.setHorizontalHeaderLabels(
            ["#", "Название", "Автор", "Альбом", "Год", "Длительность", "ID", "Обложка"]
        )
        self.table.setSelectionBehavior(QAbstractItemView.SelectionBehavior.SelectRows)
        self.table.setSelectionMode(QAbstractItemView.SelectionMode.SingleSelection)
        self.table.setEditTriggers(QAbstractItemView.EditTrigger.NoEditTriggers)
        self.table.horizontalHeader().sectionClicked.connect(self.on_header_click)
        self.table.cellDoubleClicked.connect(self.open_track_from_row)
        self.table.itemSelectionChanged.connect(self.refresh_playback_buttons)
        self.table.setContextMenuPolicy(Qt.ContextMenuPolicy.CustomContextMenu)
        self.table.customContextMenuRequested.connect(self.open_track_context_menu)
        layout.addWidget(self.table)

        self.status = QLabel("Ready")
        self.status.setWordWrap(True)
        layout.addWidget(self.status)

        playback_controls = QHBoxLayout()
        self.play_selected_button = QPushButton("Play selected")
        self.play_selected_button.clicked.connect(self.play_selected_track)
        playback_controls.addWidget(self.play_selected_button)

        self.pause_resume_button = QPushButton("Pause")
        self.pause_resume_button.clicked.connect(self.toggle_pause_playback)
        playback_controls.addWidget(self.pause_resume_button)

        self.stop_button = QPushButton("Stop")
        self.stop_button.clicked.connect(self.stop_playback)
        playback_controls.addWidget(self.stop_button)
        playback_controls.addStretch()
        layout.addLayout(playback_controls)

        self.toggle_btn = QPushButton("Показать добавление")
        self.toggle_btn.setCheckable(True)
        self.toggle_btn.clicked.connect(self.toggle_create_panel)
        layout.addWidget(self.toggle_btn)

        self.create_box = QGroupBox("Добавление")
        self.create_box.setVisible(False)
        create_layout = QVBoxLayout(self.create_box)
        create_layout.setSpacing(16)

        self.create_tabs = QTabWidget()
        self.create_tabs.addTab(self._build_author_form(), "Автор")
        self.create_tabs.addTab(self._build_album_form(), "Альбом")
        self.create_tabs.addTab(self._build_song_form(), "Трек")
        create_layout.addWidget(self.create_tabs)

        layout.addWidget(self.create_box)

    def _build_author_form(self):
        box = QGroupBox("Автор")
        layout = QVBoxLayout(box)

        form = QFormLayout()
        self.author_name = QLineEdit()
        self.author_name.setPlaceholderText("Author name")
        self.author_name.textChanged.connect(self.update_form_state)
        form.addRow("Name", self.author_name)

        self.bio = QLineEdit()
        self.bio.setPlaceholderText("Bio / description")
        self.bio.textChanged.connect(self.update_form_state)
        form.addRow("Bio", self.bio)
        layout.addLayout(form)

        buttons = QHBoxLayout()
        self.author_images_button = QPushButton("Choose images")
        self.author_images_button.clicked.connect(self.select_author_images)
        buttons.addWidget(self.author_images_button)

        clear_btn = QPushButton("Clear images")
        clear_btn.clicked.connect(self.clear_author_images)
        buttons.addWidget(clear_btn)
        buttons.addStretch()
        layout.addLayout(buttons)

        self.author_images_preview = ImagePreviewRow(6, "Author images")
        layout.addWidget(self.author_images_preview)

        self.author_error = QLabel()
        self.author_error.setWordWrap(True)
        self.author_error.setStyleSheet("color: #b91c1c;")
        layout.addWidget(self.author_error)

        self.btn_add_author = QPushButton("Добавить автора")
        self.btn_add_author.clicked.connect(self.create_author)
        layout.addWidget(self.btn_add_author, 0, Qt.AlignmentFlag.AlignLeft)

        return box

    def _build_album_form(self):
        box = QGroupBox("Альбом")
        layout = QVBoxLayout(box)

        form = QFormLayout()
        self.author_id_input = QLineEdit()
        self.author_id_input.setPlaceholderText("Author ID")
        self.author_id_input.setValidator(QIntValidator(1, 999999999, self))
        self.author_id_input.textChanged.connect(self.update_form_state)
        form.addRow("Author ID", self.author_id_input)

        self.album_name = QLineEdit()
        self.album_name.setPlaceholderText("Album name")
        self.album_name.textChanged.connect(self.update_form_state)
        form.addRow("Name", self.album_name)

        self.album_year = QLineEdit()
        self.album_year.setPlaceholderText("Year")
        self.album_year.setValidator(QIntValidator(1, 2099, self))
        self.album_year.textChanged.connect(self.update_form_state)
        form.addRow("Year", self.album_year)

        self.album_description = QLineEdit()
        self.album_description.setPlaceholderText("Description")
        self.album_description.textChanged.connect(self.update_form_state)
        form.addRow("Description", self.album_description)
        layout.addLayout(form)

        buttons = QHBoxLayout()
        self.album_cover_button = QPushButton("Choose cover")
        self.album_cover_button.clicked.connect(self.select_album_cover)
        buttons.addWidget(self.album_cover_button)

        clear_btn = QPushButton("Clear cover")
        clear_btn.clicked.connect(self.clear_album_cover)
        buttons.addWidget(clear_btn)
        buttons.addStretch()
        layout.addLayout(buttons)

        self.album_cover_preview = ImagePreviewRow(1, "Album cover")
        layout.addWidget(self.album_cover_preview)

        self.album_error = QLabel()
        self.album_error.setWordWrap(True)
        self.album_error.setStyleSheet("color: #b91c1c;")
        layout.addWidget(self.album_error)

        self.btn_add_album = QPushButton("Добавить альбом")
        self.btn_add_album.clicked.connect(self.create_album)
        layout.addWidget(self.btn_add_album, 0, Qt.AlignmentFlag.AlignLeft)

        return box

    def _build_song_form(self):
        box = QGroupBox("Песня")
        layout = QVBoxLayout(box)

        form = QFormLayout()
        self.album_id_input = QLineEdit()
        self.album_id_input.setPlaceholderText("Album ID")
        self.album_id_input.setValidator(QIntValidator(1, 999999999, self))
        self.album_id_input.textChanged.connect(self.update_form_state)
        form.addRow("Album ID", self.album_id_input)

        self.song_name_input = QLineEdit()
        self.song_name_input.setPlaceholderText("Song name")
        self.song_name_input.textChanged.connect(self.update_form_state)
        form.addRow("Name", self.song_name_input)

        self.duration_input = QLineEdit()
        self.duration_input.setPlaceholderText("Duration in seconds")
        self.duration_input.setValidator(QIntValidator(1, 999999999, self))
        self.duration_input.textChanged.connect(self.update_form_state)
        form.addRow("Duration", self.duration_input)

        self.audio_url_input = QLineEdit()
        self.audio_url_input.setPlaceholderText("Audio URL")
        self.audio_url_input.textChanged.connect(self.update_form_state)
        form.addRow("Audio URL", self.audio_url_input)
        layout.addLayout(form)

        self.song_error = QLabel()
        self.song_error.setWordWrap(True)
        self.song_error.setStyleSheet("color: #b91c1c;")
        layout.addWidget(self.song_error)

        self.btn_add_song = QPushButton("Добавить песню")
        self.btn_add_song.clicked.connect(self.create_song)
        layout.addWidget(self.btn_add_song, 0, Qt.AlignmentFlag.AlignLeft)

        return box

    def toggle_create_panel(self):
        if self.toggle_btn.isChecked():
            self.create_box.setVisible(True)
            self.toggle_btn.setText("Скрыть добавление")
        else:
            self.create_box.setVisible(False)
            self.toggle_btn.setText("Показать добавление")

    def update_form_state(self):
        author_valid, author_message = self._validate_author_form()
        self.btn_add_author.setEnabled(author_valid)
        self.author_error.setText("" if author_valid else author_message)

        album_valid, album_message = self._validate_album_form()
        self.btn_add_album.setEnabled(album_valid)
        self.album_error.setText("" if album_valid else album_message)

        song_valid, song_message = self._validate_song_form()
        self.btn_add_song.setEnabled(song_valid)
        self.song_error.setText("" if song_valid else song_message)

    def _validate_author_form(self):
        if not self.author_name.text().strip():
            return False, "Author name is required."
        if len(self.author_images_binaries) > 6:
            return False, "Author images are limited to 6 files."
        return True, ""

    def _validate_album_form(self):
        if not self.author_id_input.text().strip():
            return False, "Author ID is required."
        if not self.album_name.text().strip():
            return False, "Album name is required."
        if not self.album_year.text().strip():
            return False, "Year is required."
        return True, ""

    def _validate_song_form(self):
        if not self.album_id_input.text().strip():
            return False, "Album ID is required."
        if not self.song_name_input.text().strip():
            return False, "Song name is required."
        if not self.duration_input.text().strip():
            return False, "Duration is required."
        return True, ""

    def select_author_images(self):
        files, _ = QFileDialog.getOpenFileNames(
            self,
            "Select author images",
            "",
            "Images (*.png *.jpg *.jpeg *.bmp *.webp)",
        )
        if not files:
            return

        if len(files) > 6:
            self.show_error("You can upload up to 6 author images.")
            files = files[:6]

        self.author_images_binaries = [self.file_to_base64(path) for path in files]
        self.author_images_preview.set_images(self.author_images_binaries)
        self.update_form_state()

    def clear_author_images(self):
        self.author_images_binaries = []
        self.author_images_preview.clear()
        self.update_form_state()

    def select_album_cover(self):
        file_path, _ = QFileDialog.getOpenFileName(
            self,
            "Select album cover",
            "",
            "Images (*.png *.jpg *.jpeg *.bmp *.webp)",
        )
        if not file_path:
            return

        self.album_cover_binary = self.file_to_base64(file_path)
        self.album_cover_preview.set_images([self.album_cover_binary])
        self.update_form_state()

    def clear_album_cover(self):
        self.album_cover_binary = ""
        self.album_cover_preview.clear()
        self.update_form_state()

    def file_to_base64(self, path):
        with open(path, "rb") as file_obj:
            return base64.b64encode(file_obj.read()).decode("utf-8")

    def set_status(self, message):
        self.status.setText(message)

    def is_current_track(self, track):
        if not track or not self.current_playing_track:
            return False
        return track.get("song_id") == self.current_playing_track.get("song_id")

    def refresh_playback_buttons(self):
        has_current = self.current_playing_track is not None
        self.pause_resume_button.setEnabled(has_current)
        self.stop_button.setEnabled(has_current)
        self.pause_resume_button.setText("Resume" if self.playback_paused else "Pause")
        self.play_selected_button.setEnabled(self.table.currentRow() >= 0)
        self.track_page.refresh_playback_state()

    def show_error(self, message, title="Error"):
        self.set_status(message)
        QMessageBox.warning(self, title, message)

    def show_info(self, message, title="Info"):
        self.set_status(message)
        QMessageBox.information(self, title, message)

    def request_json(self, payload):
        response = requests.post(SERVER_URL, json=payload, timeout=REQUEST_TIMEOUT)
        response.raise_for_status()
        result = response.json()
        if result.get("error"):
            raise ValueError(result.get("error_message") or "API returned an error")
        return result

    def fetch_items(self, entity, filters=None, sort=None):
        result = self.request_json(
            {
                "method": "read",
                "entity": entity,
                "filters": filters or {},
                "sort": sort or {"field": None, "order": None},
            }
        )
        return result.get("data", {}).get("items", []) or []

    def find_exact(self, items, key, value):
        value = (value or "").strip().lower()
        for item in items:
            current = str(item.get(key, "")).strip().lower()
            if current == value:
                return item
        return items[0] if items else None

    def get_pixmap(self, track):
        cover_binary = track.get("cover_binary")
        pixmap = decode_pixmap(cover_binary)
        if pixmap:
            return pixmap

        url = track.get("url")
        if url:
            try:
                response = requests.get(url, timeout=2)
                response.raise_for_status()
                pixmap = QPixmap()
                pixmap.loadFromData(response.content)
                if not pixmap.isNull():
                    return pixmap
            except Exception:
                pass

        if os.path.exists(PLACEHOLDER_PATH):
            return QPixmap(PLACEHOLDER_PATH)
        return None

    def set_image_label(self, label, binary, width, height):
        pixmap = decode_pixmap(binary)
        if not pixmap and os.path.exists(PLACEHOLDER_PATH):
            pixmap = QPixmap(PLACEHOLDER_PATH)

        label.setFixedSize(width, height)
        if pixmap and not pixmap.isNull():
            label.setPixmap(
                pixmap.scaled(
                    width,
                    height,
                    Qt.AspectRatioMode.KeepAspectRatio,
                    Qt.TransformationMode.SmoothTransformation,
                )
            )
            label.setText("")
        else:
            label.setPixmap(QPixmap())
            label.setText("No image")

    def load_all_tracks(self):
        self.search_with_sort(None, None)

    def search(self):
        self.search_with_sort(None, None)

    def search_with_sort(self, field, order):
        filters = {}

        if self.song_id.text():
            value = parse_range(self.song_id.text())
            if value:
                filters["song_id"] = value

        if self.author.text():
            filters["author"] = self.author.text().strip()

        if self.song.text():
            filters["song_name"] = self.song.text().strip()

        if self.album_search.text():
            filters["album_name"] = self.album_search.text().strip()

        if self.year.text():
            value = parse_range(self.year.text())
            if value:
                filters["year"] = value

        if self.duration.text():
            try:
                filters["duration_max"] = int(self.duration.text())
            except ValueError:
                pass

        try:
            self.tracks = self.fetch_items(
                "song",
                filters=filters,
                sort={"field": field, "order": order},
            )
            self.fill_table(self.tracks)
            self.set_status(f"Найдено: {len(self.tracks)}")
        except (requests.RequestException, ValueError) as err:
            self.show_error(str(err))

    def on_header_click(self, index):
        columns = ["index", "song_name", "author", "album_name", "year", "duration_sec", "song_id"]
        if index >= len(columns):
            return

        field = columns[index]
        order = self.sort_state.get(field, "asc")
        order = "desc" if order == "asc" else "asc"
        self.sort_state[field] = order
        self.search_with_sort(field, order)

    def fill_table(self, tracks):
        self.table.setRowCount(len(tracks))

        for row, track in enumerate(tracks):
            self.table.setRowHeight(row, 70)

            self.table.setItem(row, 0, QTableWidgetItem(str(row + 1)))
            self.table.setItem(row, 1, QTableWidgetItem(str(track.get("song_name", ""))))
            self.table.setItem(row, 2, QTableWidgetItem(str(track.get("author", ""))))
            self.table.setItem(row, 3, QTableWidgetItem(str(track.get("album_name", ""))))
            self.table.setItem(row, 4, QTableWidgetItem(str(track.get("year", ""))))
            self.table.setItem(row, 5, QTableWidgetItem(str(track.get("duration_sec", ""))))
            self.table.setItem(row, 6, QTableWidgetItem(str(track.get("song_id", ""))))

            pixmap = self.get_pixmap(track)
            if pixmap:
                label = QLabel()
                label.setAlignment(Qt.AlignmentFlag.AlignCenter)
                label.setPixmap(
                    pixmap.scaled(
                        60,
                        60,
                        Qt.AspectRatioMode.KeepAspectRatio,
                        Qt.TransformationMode.SmoothTransformation,
                    )
                )
                self.table.setCellWidget(row, 7, label)
            else:
                self.table.setCellWidget(row, 7, None)

        self.refresh_playback_buttons()

    def track_for_row(self, row):
        if 0 <= row < len(self.tracks):
            return self.tracks[row]
        return None

    def open_track_from_row(self, row, _column):
        track = self.track_for_row(row)
        if track:
            self.open_track_page(track.get("song_id"))

    def play_selected_track(self):
        row = self.table.currentRow()
        track = self.track_for_row(row)
        if not track:
            self.show_info("Select a track in the table first.")
            return
        self.play_audio(track)

    def open_track_context_menu(self, pos):
        index = self.table.indexAt(pos)
        if not index.isValid():
            return

        row = index.row()
        track = self.track_for_row(row)
        if not track:
            return

        self.table.selectRow(row)

        menu = QMenu(self)
        author_action = QAction("Author", self)
        album_action = QAction("Album", self)
        track_action = QAction("Track", self)

        author_action.triggered.connect(lambda: self.open_author_page(track.get("author")))
        album_action.triggered.connect(
            lambda: self.open_album_page(
                album_name=track.get("album_name"),
                author_name=track.get("author"),
            )
        )
        track_action.triggered.connect(lambda: self.open_track_page(track.get("song_id")))

        menu.addAction(author_action)
        menu.addAction(album_action)
        menu.addAction(track_action)
        menu.exec(self.table.viewport().mapToGlobal(pos))

    def _set_current_page(self, page, title, remember=True):
        current = self.stack.currentWidget()
        if remember and current is not page:
            self.navigation_history.append((current, self.page_title.text()))

        self.stack.setCurrentWidget(page)
        self.page_title.setText(title)
        self._update_navigation_ui()

    def _update_navigation_ui(self):
        on_search = self.stack.currentWidget() is self.search_page
        self.back_button.setVisible(not on_search)

    def go_back(self):
        if not self.navigation_history:
            self.stack.setCurrentWidget(self.search_page)
            self.page_title.setText("Tracks")
            self._update_navigation_ui()
            return

        page, title = self.navigation_history.pop()
        self.stack.setCurrentWidget(page)
        self.page_title.setText(title)
        self._update_navigation_ui()

    def open_author_page(self, author_name):
        if not author_name:
            self.show_error("Author name is missing.")
            return

        try:
            authors = self.fetch_items("author", filters={"author": author_name})
            author = self.find_exact(authors, "author", author_name)
            if not author:
                raise ValueError("Author was not found.")

            albums = self.fetch_items(
                "album",
                filters={"author": author.get("author")},
                sort={"field": "year", "order": "asc"},
            )
            tracks = self.fetch_items(
                "song",
                filters={"author": author.get("author")},
                sort={"field": "song_name", "order": "asc"},
            )
            self.author_page.set_content(author, albums, tracks)
            self._set_current_page(self.author_page, author.get("author") or "Author")
        except (requests.RequestException, ValueError) as err:
            self.show_error(str(err))

    def resolve_album(self, album_id=None, album_name=None, author_name=None):
        filters = {}
        if album_id is not None:
            filters["album_id"] = {"min": int(album_id), "max": int(album_id)}
        if album_name:
            filters["album_name"] = album_name
        if author_name:
            filters["author"] = author_name

        albums = self.fetch_items("album", filters=filters)
        if album_id is not None:
            for album in albums:
                if int(album.get("album_id", -1)) == int(album_id):
                    return album
        if album_name:
            for album in albums:
                same_name = str(album.get("album_name", "")).strip().lower() == album_name.strip().lower()
                same_author = not author_name or str(album.get("author", "")).strip().lower() == author_name.strip().lower()
                if same_name and same_author:
                    return album
        return albums[0] if albums else None

    def open_album_page(self, album_id=None, album_name=None, author_name=None):
        try:
            album = self.resolve_album(album_id=album_id, album_name=album_name, author_name=author_name)
            if not album:
                raise ValueError("Album was not found.")

            tracks = self.fetch_items(
                "song",
                filters={
                    "author": album.get("author"),
                    "album_name": album.get("album_name"),
                },
                sort={"field": "song_name", "order": "asc"},
            )
            self.album_page.set_content(album, tracks)
            self._set_current_page(self.album_page, album.get("album_name") or "Album")
        except (requests.RequestException, ValueError) as err:
            self.show_error(str(err))

    def resolve_track(self, song_id):
        songs = self.fetch_items("song", filters={"song_id": {"min": int(song_id), "max": int(song_id)}})
        for song in songs:
            if int(song.get("song_id", -1)) == int(song_id):
                return song
        return songs[0] if songs else None

    def open_track_page(self, song_id):
        if song_id is None:
            self.show_error("Track id is missing.")
            return

        try:
            track = self.resolve_track(song_id)
            if not track:
                raise ValueError("Track was not found.")

            album = self.resolve_album(
                album_name=track.get("album_name"),
                author_name=track.get("author"),
            )
            authors = self.fetch_items("author", filters={"author": track.get("author")})
            author = self.find_exact(authors, "author", track.get("author"))

            self.track_page.set_content(track, album, author)
            self._set_current_page(self.track_page, track.get("song_name") or "Track")
        except (requests.RequestException, ValueError) as err:
            self.show_error(str(err))

    def play_audio(self, track):
        audio_url = track.get("audio_url")
        if not audio_url:
            self.show_info("This track does not have an audio URL.")
            return

        try:
            self.stop_playback(clear_status=False)
            response = requests.get(audio_url, timeout=REQUEST_TIMEOUT)
            response.raise_for_status()

            suffix = os.path.splitext(audio_url)[1] or ".mp3"
            temp_file = tempfile.NamedTemporaryFile(delete=False, suffix=suffix)
            temp_file.write(response.content)
            temp_file.close()

            self.temp_audio_file = temp_file.name
            pygame.mixer.music.load(self.temp_audio_file)
            pygame.mixer.music.play()
            self.current_playing_track = track
            self.playback_paused = False
            self.refresh_playback_buttons()
            self.set_status(f'Playing: {track.get("song_name", "track")}')
        except Exception as err:
            self.show_error(f"Unable to play audio: {err}")

    def toggle_pause_playback(self):
        if not self.current_playing_track:
            self.show_info("No track is currently loaded.")
            return

        try:
            if self.playback_paused:
                pygame.mixer.music.unpause()
                self.playback_paused = False
                self.set_status(f'Resumed: {self.current_playing_track.get("song_name", "track")}')
            else:
                pygame.mixer.music.pause()
                self.playback_paused = True
                self.set_status(f'Paused: {self.current_playing_track.get("song_name", "track")}')
            self.refresh_playback_buttons()
        except Exception as err:
            self.show_error(f"Unable to change playback state: {err}")

    def stop_playback(self, clear_status=True):
        try:
            pygame.mixer.music.stop()
        except Exception:
            pass

        self.playback_paused = False
        self.current_playing_track = None
        self.refresh_playback_buttons()
        if clear_status:
            self.set_status("Playback stopped")

    def create_author(self):
        valid, message = self._validate_author_form()
        if not valid:
            self.show_error(message)
            return

        try:
            result = self.request_json(
                {
                    "method": "write",
                    "entity": "author",
                    "data": {
                        "author_name": self.author_name.text().strip(),
                        "bio": self.bio.text().strip(),
                        "images_binaries": self.author_images_binaries,
                    },
                }
            )
            created_id = result.get("created_id")
            if created_id is not None:
                self.author_id_input.setText(str(created_id))

            self.author_name.clear()
            self.bio.clear()
            self.clear_author_images()
            self.set_status(f"Author created with id {created_id}")
        except (requests.RequestException, ValueError) as err:
            self.show_error(str(err))

    def create_album(self):
        valid, message = self._validate_album_form()
        if not valid:
            self.show_error(message)
            return

        try:
            result = self.request_json(
                {
                    "method": "write",
                    "entity": "album",
                    "data": {
                        "author_id": int(self.author_id_input.text()),
                        "album_name": self.album_name.text().strip(),
                        "year": int(self.album_year.text()),
                        "description": self.album_description.text().strip(),
                        "cover_binary": self.album_cover_binary,
                    },
                }
            )
            created_id = result.get("created_id")
            if created_id is not None:
                self.album_id_input.setText(str(created_id))

            self.album_name.clear()
            self.album_year.clear()
            self.album_description.clear()
            self.clear_album_cover()
            self.set_status(f"Album created with id {created_id}")
        except (requests.RequestException, ValueError) as err:
            self.show_error(str(err))

    def create_song(self):
        valid, message = self._validate_song_form()
        if not valid:
            self.show_error(message)
            return

        try:
            result = self.request_json(
                {
                    "method": "write",
                    "entity": "song",
                    "data": {
                        "album_id": int(self.album_id_input.text()),
                        "song_name": self.song_name_input.text().strip(),
                        "duration_sec": int(self.duration_input.text()),
                        "audio_url": self.audio_url_input.text().strip(),
                    },
                }
            )
            self.song_name_input.clear()
            self.duration_input.clear()
            self.audio_url_input.clear()
            self.load_all_tracks()
            self.set_status(f'Song created with id {result.get("created_id")}')
        except (requests.RequestException, ValueError) as err:
            self.show_error(str(err))


if __name__ == "__main__":
    app = QApplication(sys.argv)
    window = App()
    window.show()
    sys.exit(app.exec())
