# audiometa

> Fork of [audiotags](https://github.com/TianyiShi2001/audiotags) — the original project is no longer maintained.

This crate makes it easier to parse, convert and write metadata (a.k.a. tag) in audio files of different file types.

This crate aims to provide a unified trait for parsers and writers of different audio file formats.
This means that you can parse tags in mp3, flac, and m4a files with a single function: `Tag::default().
read_from_path()` and get fields by directly calling `.album()`, `.artist()` on its result. Without this
crate, you would otherwise need to learn different APIs in **id3**, **mp4ameta** etc. in order to parse
metadata in different file formats.

### Performance

Using **audiometa** incurs a little overhead due to vtables if you want to guess the metadata format
(from file extension). Apart from this the performance is almost the same as directly calling function
provided by those 'specialized' crates.

No copies will be made if you only need to read and write metadata of one format. If you want to convert
between tags, copying is unavoidable no matter if you use **audiometa** or use getters and setters provided
by specialized libraries. **audiometa** is not making additional unnecessary copies.

### Supported Formats

| File Format   | Metadata Format       | backend                                                     |
|---------------|-----------------------|-------------------------------------------------------------|
| `mp3`         | id3v2.4               | [**id3**](https://github.com/polyfloyd/rust-id3)            |
| `m4a/mp4/...` | MPEG-4 audio metadata | [**mp4ameta**](https://github.com/Saecki/rust-mp4ameta)     |
| `flac`        | Vorbis comment        | [**metaflac**](https://github.com/jameshurst/rust-metaflac) |

### Examples

```rust
fn main() {
    // using `default()` or `new()` alone so that the metadata format is
    // guessed (from the file extension) (in this case, Id3v2 tag is read)
    let mut tag = Tag::new().read_from_path(MP3_FILE).unwrap();

    tag.set_title("foo title");
    assert_eq!(tag.title(), Some("foo title"));
    tag.remove_title();
    assert!(tag.title().is_none());
    tag.remove_title();
    // trying to remove a field that's already empty won't hurt

    let cover = Picture {
        mime_type: MimeType::Jpeg,
        data: &vec![0u8; 10],
    };

    tag.set_album_cover(cover.clone());
    assert_eq!(tag.album_cover(), Some(cover));
    tag.remove_album_cover();
    assert!(tag.album_cover().is_none());
    tag.remove_album_cover();

    tag.write_to_path(MP3_FILE).expect("Fail to save");
}
```

License: MIT
