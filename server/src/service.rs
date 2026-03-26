use std::cmp::Ordering;
use std::sync::Arc;

use saod::a2_tree::A2Tree;
use saod::binary_search::{binary_search_by, equal_range_by};
use saod::digital_sort::{
    radix_sort_by_selected_field, SortDirection as SaodSortDirection, SortField as SaodSortField,
};

use crate::error::{ApiError, ErrorCode};
use crate::models::{
    AlbumRecord, ApiRequest, ApiResponse, AuthorPayload, Entity, Filters, RangeFilter,
    ResponseData, SongRecord, Sort, SortField, SortOrder,
};
use crate::repo::MediaRepository;

#[derive(Clone)]
pub struct SearchService {
    repo: Arc<dyn MediaRepository>,
}

impl SearchService {
    pub fn new(repo: Arc<dyn MediaRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, request: ApiRequest) -> Result<ApiResponse, ApiError> {
        validate_request(&request)?;

        match request.entity {
            Entity::Song => self.search_songs(request).await,
            Entity::Album => self.search_albums(request).await,
            Entity::Author => self.search_author(request).await,
        }
    }

    async fn search_songs(&self, request: ApiRequest) -> Result<ApiResponse, ApiError> {
        let mut songs = self
            .repo
            .all_songs()
            .await
            .map_err(ApiError::from_repo_error)?
            .into_iter()
            .filter(|song| apply_song_filters(song, &request.filters))
            .collect::<Vec<_>>();

        apply_song_sort(&mut songs, &request.sort)?;
        apply_song_binary_search_hint(&songs, &request.filters, &request.sort);

        if should_build_a2_tree() {
            let mut tree = A2Tree::new();
            let field = request.sort.field.as_ref().unwrap_or(&SortField::Year);
            let order = request.sort.order.as_ref().unwrap_or(&SortOrder::Asc);
            for item in songs.iter().cloned() {
                tree.insert_by(item, |a, b| song_cmp(a, b, field, order));
            }
            let _tree_size = tree.len();
        }

        if songs.is_empty() {
            return Err(ApiError::not_found(
                "Songs were not found by provided filters",
            ));
        }

        Ok(ok_response(request, ResponseData::Song(songs)))
    }

    async fn search_albums(&self, request: ApiRequest) -> Result<ApiResponse, ApiError> {
        let mut albums = self
            .repo
            .all_albums()
            .await
            .map_err(ApiError::from_repo_error)?
            .into_iter()
            .filter(|album| apply_album_filters(album, &request.filters))
            .collect::<Vec<_>>();

        apply_album_sort(&mut albums, &request.sort)?;
        apply_album_binary_search_hint(&albums, &request.filters, &request.sort);

        if should_build_a2_tree() {
            let mut tree = A2Tree::new();
            let field = request.sort.field.as_ref().unwrap_or(&SortField::Year);
            let order = request.sort.order.as_ref().unwrap_or(&SortOrder::Asc);
            for item in albums.iter().cloned() {
                tree.insert_by(item, |a, b| album_cmp(a, b, field, order));
            }
            let _tree_size = tree.len();
        }

        if albums.is_empty() {
            return Err(ApiError::not_found(
                "Albums were not found by provided filters",
            ));
        }

        Ok(ok_response(request, ResponseData::Album(albums)))
    }

    async fn search_author(&self, request: ApiRequest) -> Result<ApiResponse, ApiError> {
        let author = request.filters.author.clone().ok_or_else(|| {
            ApiError::invalid_request("For entity=author, filters.author is required")
        })?;

        let albums = self
            .repo
            .all_albums()
            .await
            .map_err(ApiError::from_repo_error)?
            .into_iter()
            .filter(|a| a.author.eq_ignore_ascii_case(&author))
            .collect::<Vec<_>>();

        let mut images = self
            .repo
            .author_images(&author)
            .await
            .map_err(ApiError::from_repo_error)?;
        images.truncate(6);

        if albums.is_empty() && images.is_empty() {
            return Err(ApiError::not_found("Author media was not found"));
        }

        let payload = AuthorPayload {
            author,
            albums,
            images,
        };

        Ok(ok_response(request, ResponseData::Author(payload)))
    }
}

fn should_build_a2_tree() -> bool {
    std::env::var("ENABLE_A2_TREE")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn apply_song_binary_search_hint(songs: &[SongRecord], filters: &Filters, sort: &Sort) {
    if songs.is_empty() {
        return;
    }

    if matches!(sort.field, Some(SortField::Author)) {
        if let Some(author) = &filters.author {
            let probe = SongRecord {
                song_id: 0,
                author: author.clone(),
                album_name: String::new(),
                song_name: String::new(),
                year: 0,
                duration_sec: 0,
            };
            let cmp = |a: &SongRecord, b: &SongRecord| {
                a.author
                    .to_ascii_lowercase()
                    .cmp(&b.author.to_ascii_lowercase())
            };
            if binary_search_by(songs, &probe, cmp).is_some() {
                let _range = equal_range_by(songs, &probe, cmp);
            }
        }
    }
}

fn apply_album_binary_search_hint(albums: &[AlbumRecord], filters: &Filters, sort: &Sort) {
    if albums.is_empty() {
        return;
    }

    if matches!(sort.field, Some(SortField::Author)) {
        if let Some(author) = &filters.author {
            let probe = AlbumRecord {
                album_id: 0,
                author: author.clone(),
                album_name: String::new(),
                description: String::new(),
                cover_url: String::new(),
                year: 0,
            };
            let cmp = |a: &AlbumRecord, b: &AlbumRecord| {
                a.author
                    .to_ascii_lowercase()
                    .cmp(&b.author.to_ascii_lowercase())
            };
            if binary_search_by(albums, &probe, cmp).is_some() {
                let _range = equal_range_by(albums, &probe, cmp);
            }
        }
    }
}

fn ok_response(request: ApiRequest, data: ResponseData) -> ApiResponse {
    ApiResponse {
        entity: Some(request.entity),
        filters: request.filters,
        sort: request.sort,
        error: ErrorCode::None.as_u8(),
        error_message: None,
        data: Some(data),
    }
}

pub fn error_response(request: ApiRequest, err: ApiError) -> ApiResponse {
    ApiResponse {
        entity: Some(request.entity),
        filters: request.filters,
        sort: request.sort,
        error: err.code.as_u8(),
        error_message: Some(err.message),
        data: None,
    }
}

fn validate_range(name: &str, range: &RangeFilter) -> Result<(), ApiError> {
    if range.min > range.max {
        return Err(ApiError::invalid_request(format!(
            "Invalid range in filters.{name}: min must be <= max"
        )));
    }
    Ok(())
}

fn validate_request(request: &ApiRequest) -> Result<(), ApiError> {
    if let Some(range) = &request.filters.song_id {
        validate_range("song_id", range)?;
    }
    if let Some(range) = &request.filters.album_id {
        validate_range("album_id", range)?;
    }
    if let Some(range) = &request.filters.year {
        validate_range("year", range)?;
    }

    let sort_is_partial = request.sort.field.is_some() ^ request.sort.order.is_some();
    if sort_is_partial {
        return Err(ApiError::invalid_request(
            "sort.field and sort.order must be both null or both specified",
        ));
    }

    match request.entity {
        Entity::Song => validate_song_sort(&request.sort),
        Entity::Album => validate_album_sort(&request.sort),
        Entity::Author => {
            if request.sort.field.is_some() || request.sort.order.is_some() {
                return Err(ApiError::invalid_request(
                    "entity=author does not support sorting",
                ));
            }
            Ok(())
        }
    }
}

fn validate_song_sort(sort: &Sort) -> Result<(), ApiError> {
    match sort.field {
        Some(SortField::Year | SortField::SongName | SortField::AlbumName | SortField::Author)
        | None => Ok(()),
    }
}

fn validate_album_sort(sort: &Sort) -> Result<(), ApiError> {
    match sort.field {
        Some(SortField::Year | SortField::AlbumName | SortField::Author) | None => Ok(()),
        Some(SortField::SongName) => Err(ApiError::invalid_request(
            "song_name sorting is not supported for entity=album",
        )),
    }
}

fn apply_song_filters(song: &SongRecord, f: &Filters) -> bool {
    if let Some(range) = &f.song_id {
        if song.song_id < range.min || song.song_id > range.max {
            return false;
        }
    }
    if let Some(author) = &f.author {
        if !song.author.eq_ignore_ascii_case(author) {
            return false;
        }
    }
    if let Some(album_name) = &f.album_name {
        if !song.album_name.eq_ignore_ascii_case(album_name) {
            return false;
        }
    }
    if let Some(song_name) = &f.song_name {
        if !song.song_name.eq_ignore_ascii_case(song_name) {
            return false;
        }
    }
    if let Some(range) = &f.year {
        if song.year < range.min || song.year > range.max {
            return false;
        }
    }
    if let Some(max_duration) = f.duration_max {
        if song.duration_sec > max_duration {
            return false;
        }
    }
    true
}

fn apply_album_filters(album: &AlbumRecord, f: &Filters) -> bool {
    if let Some(range) = &f.album_id {
        if album.album_id < range.min || album.album_id > range.max {
            return false;
        }
    }
    if let Some(author) = &f.author {
        if !album.author.eq_ignore_ascii_case(author) {
            return false;
        }
    }
    if let Some(album_name) = &f.album_name {
        if !album.album_name.eq_ignore_ascii_case(album_name) {
            return false;
        }
    }
    if let Some(range) = &f.year {
        if album.year < range.min || album.year > range.max {
            return false;
        }
    }
    true
}

fn to_saod_direction(order: &SortOrder) -> SaodSortDirection {
    match order {
        SortOrder::Asc => SaodSortDirection::Asc,
        SortOrder::Desc => SaodSortDirection::Desc,
    }
}

fn apply_song_sort(songs: &mut [SongRecord], sort: &Sort) -> Result<(), ApiError> {
    let Some(field) = &sort.field else {
        return Ok(());
    };
    let Some(order) = &sort.order else {
        return Ok(());
    };

    let dir = to_saod_direction(order);

    match field {
        SortField::Year => radix_sort_by_selected_field(
            songs,
            SaodSortField::Year,
            dir,
            |x| x.song_id as u64,
            |x| &x.author,
            |x| &x.song_name,
            |x| x.year as u64,
        ),
        SortField::SongName => radix_sort_by_selected_field(
            songs,
            SaodSortField::Name,
            dir,
            |x| x.song_id as u64,
            |x| &x.author,
            |x| &x.song_name,
            |x| x.year as u64,
        ),
        SortField::AlbumName => radix_sort_by_selected_field(
            songs,
            SaodSortField::Name,
            dir,
            |x| x.song_id as u64,
            |x| &x.author,
            |x| &x.album_name,
            |x| x.year as u64,
        ),
        SortField::Author => radix_sort_by_selected_field(
            songs,
            SaodSortField::Author,
            dir,
            |x| x.song_id as u64,
            |x| &x.author,
            |x| &x.song_name,
            |x| x.year as u64,
        ),
    }

    Ok(())
}

fn apply_album_sort(albums: &mut [AlbumRecord], sort: &Sort) -> Result<(), ApiError> {
    let Some(field) = &sort.field else {
        return Ok(());
    };
    let Some(order) = &sort.order else {
        return Ok(());
    };

    let dir = to_saod_direction(order);

    match field {
        SortField::Year => radix_sort_by_selected_field(
            albums,
            SaodSortField::Year,
            dir,
            |x| x.album_id as u64,
            |x| &x.author,
            |x| &x.album_name,
            |x| x.year as u64,
        ),
        SortField::AlbumName => radix_sort_by_selected_field(
            albums,
            SaodSortField::Name,
            dir,
            |x| x.album_id as u64,
            |x| &x.author,
            |x| &x.album_name,
            |x| x.year as u64,
        ),
        SortField::Author => radix_sort_by_selected_field(
            albums,
            SaodSortField::Author,
            dir,
            |x| x.album_id as u64,
            |x| &x.author,
            |x| &x.album_name,
            |x| x.year as u64,
        ),
        SortField::SongName => {
            return Err(ApiError::invalid_request(
                "song_name sorting is not supported for entity=album",
            ));
        }
    }

    Ok(())
}

fn song_cmp(a: &SongRecord, b: &SongRecord, field: &SortField, order: &SortOrder) -> Ordering {
    let cmp = match field {
        SortField::Year => a.year.cmp(&b.year),
        SortField::SongName => a.song_name.cmp(&b.song_name),
        SortField::AlbumName => a.album_name.cmp(&b.album_name),
        SortField::Author => a.author.cmp(&b.author),
    };

    match order {
        SortOrder::Asc => cmp,
        SortOrder::Desc => reverse(cmp),
    }
}

fn album_cmp(
    a: &AlbumRecord,
    b: &AlbumRecord,
    field: &SortField,
    order: &SortOrder,
) -> Ordering {
    let cmp = match field {
        SortField::Year => a.year.cmp(&b.year),
        SortField::AlbumName => a.album_name.cmp(&b.album_name),
        SortField::Author => a.author.cmp(&b.author),
        SortField::SongName => Ordering::Equal,
    };

    match order {
        SortOrder::Asc => cmp,
        SortOrder::Desc => reverse(cmp),
    }
}

fn reverse(cmp: Ordering) -> Ordering {
    match cmp {
        Ordering::Less => Ordering::Greater,
        Ordering::Equal => Ordering::Equal,
        Ordering::Greater => Ordering::Less,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::models::{ApiRequest, Entity, Filters, RangeFilter, Sort, SortField, SortOrder};
    use crate::repo::InMemoryRepository;

    use super::SearchService;

    #[tokio::test]
    async fn song_filter_and_sort_desc_year_works() {
        let service = SearchService::new(Arc::new(InMemoryRepository::new_seeded()));
        let req = ApiRequest {
            entity: Entity::Song,
            filters: Filters {
                song_id: Some(RangeFilter { min: 10, max: 72 }),
                author: Some("Linkin Park".to_string()),
                year: Some(RangeFilter {
                    min: 2000,
                    max: 2010,
                }),
                duration_max: Some(300),
                ..Filters::default()
            },
            sort: Sort {
                field: Some(SortField::Year),
                order: Some(SortOrder::Desc),
            },
        };

        let out = service.execute(req).await.expect("must work");
        let data = out.data.expect("must have data");

        match data {
            crate::models::ResponseData::Song(songs) => {
                assert_eq!(songs.len(), 2);
                assert_eq!(songs[0].year, 2007);
                assert_eq!(songs[1].year, 2003);
            }
            _ => panic!("wrong response type"),
        }
    }

    #[tokio::test]
    async fn invalid_range_returns_error() {
        let service = SearchService::new(Arc::new(InMemoryRepository::new_seeded()));
        let req = ApiRequest {
            entity: Entity::Song,
            filters: Filters {
                year: Some(RangeFilter {
                    min: 2010,
                    max: 2000,
                }),
                ..Filters::default()
            },
            sort: Sort::default(),
        };

        let err = service.execute(req).await.expect_err("must fail");
        assert_eq!(err.code as u8, 4);
    }

    #[tokio::test]
    async fn author_images_are_limited_to_six() {
        let service = SearchService::new(Arc::new(InMemoryRepository::new_seeded()));
        let req = ApiRequest {
            entity: Entity::Author,
            filters: Filters {
                author: Some("Linkin Park".to_string()),
                ..Filters::default()
            },
            sort: Sort::default(),
        };

        let out = service.execute(req).await.expect("must work");

        match out.data.expect("must have data") {
            crate::models::ResponseData::Author(payload) => {
                assert_eq!(payload.images.len(), 6);
                assert!(!payload.albums.is_empty());
            }
            _ => panic!("wrong response type"),
        }
    }
}
