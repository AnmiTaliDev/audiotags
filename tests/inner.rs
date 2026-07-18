use audiometa::*;
use std::fs;
use tempfile::Builder;

#[test]
fn test_inner() {
    let tmp = Builder::new().suffix(".mp3").tempfile().unwrap();
    fs::copy("assets/a.mp3", &tmp).unwrap();

    let tmp_path = tmp.path();

    let mut innertag = FlacInnerTag::default();
    let title = "title from FlacInnerTag";
    let artist = "Billy Foo";
    let album_artist = "Billy Foo & The Bars";
    innertag.vorbis_comments_mut().set("TITLE", title);
    innertag.vorbis_comments_mut().set("ARTIST", artist);
    innertag
        .vorbis_comments_mut()
        .set("ALBUMARTIST", album_artist);

    let tag: FlacTag = innertag.into();
    let mut id3tag = tag.to_dyn_tag(TagType::Id3v2);

    id3tag
        .write_to_path(tmp_path.to_str().unwrap())
        .expect("Fail to write!");

    let id3tag_reload = Tag::default()
        .read_from_path(tmp_path)
        .expect("Fail to read!");

    assert_eq!(id3tag_reload.title(), Some(title));
    assert_eq!(id3tag_reload.artist(), Some(artist));
    assert_eq!(id3tag_reload.album_artist(), Some(album_artist));

    let mut id3tag_inner: Id3v2InnerTag = id3tag_reload.into();
    let timestamp = Timestamp {
        year: 2013,
        month: Some(2u8),
        day: Some(5u8),
        hour: Some(6u8),
        minute: None,
        second: None,
    };

    id3tag_inner.set_date_recorded(timestamp);
    id3tag_inner
        .write_to_path(tmp_path)
        .expect("Fail to write!");

    let id3tag_reload = Id3v2InnerTag::read_from_path(tmp_path).expect("Fail to read!");
    assert_eq!(id3tag_reload.date_recorded(), Some(timestamp));
    assert_eq!(id3tag_reload.artist(), Some(artist));
    assert_eq!(id3tag_reload.album_artist(), Some(album_artist));
}
