# MIDI library

The home screen is a folder-backed gallery of your MIDI pieces.

1. Click **Choose folder** and select the folder where you keep your music.
2. Neothesia saves the folder immediately and lists `.mid` and `.midi` files,
   including upper-case extensions. Subfolders are included by default.
3. Click a piece's card to load it and start playback. **Escape** returns to the
   gallery; **Tracks** and **Play** in the bottom bar remain available for the
   last selected piece.

The folder and subfolder preference survive app restarts. The folder is scanned
when the menu is created, including at startup and when returning from playback
or Free Play. Files remain in their original locations; Neothesia does not copy,
move or delete them. **Change folder** replaces the saved folder; cancelling the
picker keeps the current one.

## Finding a piece

- Search by title or relative folder path. Search ignores case, supports Unicode
  names, and matches every word you enter. `bach prelude` finds Prelude files in
  a Bach subfolder. Click the search field or press **Ctrl+L** (**Cmd+L** on macOS).
- **Ctrl+A / Cmd+A** selects the whole search for replacement. Backspace edits
  the query, **Clear** resets it, and **Enter** plays the first matching piece.
  Escape first leaves search focus. The existing Ctrl+F fullscreen shortcut is
  unchanged; F11 also toggles fullscreen.
- **Sort** switches between name and newest modification time. Equal dates keep
  name order; files without a modification time sort last.
- Cards show the filename without its extension, the relative parent folder and
  file size. Identical titles in different subfolders remain separate entries.
- Previous/Next pages adapt to window size, so large catalogs remain usable.
  Only the current page's cards are rendered. Search and sorting cover the full
  catalog, not just the displayed page.
- **Select File** still opens a file anywhere without changing the library folder.
  **FreePlay** remains available from the bottom bar.

## Updating and unavailable files

Use **Refresh** after adding, removing or renaming files while the app is open.
This version rescans on startup/menu entry and on request; it does not watch the
folder continuously. Toggle **Subfolders** to choose whether nested folders are
included. This preference saves immediately and triggers a rescan.

Scanning runs on a background thread and reads paths and filesystem metadata,
not every MIDI file's contents. MIDI parsing also runs in the background when a
piece is opened. A malformed file reports an error in the gallery, while other
pieces remain usable. A missing/deleted file behaves the same way until Refresh
removes it from the list.

An unavailable library folder (for example an unplugged drive) shows an error
and keeps the saved folder, so reconnecting and pressing Refresh can recover.
Unreadable descendant entries are skipped with a visible count. Descendant
file/directory symlinks are not followed, preventing cycles and unintended
traversal into linked folders. Normal folders can be used as the selected root.
The catalog is rebuilt from disk, so stale cached entries are not restored.

The folder is stored as `history.V1.midi_folder` and recursion as
`history.V1.include_subfolders` in the existing settings file. Old settings default
to no folder and included subfolders, preserving the last-opened song and other
preferences. Search, sort and page position belong to the current menu session.

## Verification

Automated coverage checks mixed extensions, recursive/nonrecursive listing,
Unicode and multiword/folder search, duplicate names, stable sorting, added,
renamed and removed files, unavailable roots, symlink cycles on Unix, and old
settings plus preference serialization round trips.

Desktop checks:

1. Choose a folder with both root and nested MIDI files, restart, and confirm the
   same folder and all files reappear without another picker.
2. Search, clear and sort; check duplicate names in different folders. Use enough
   files for multiple pages, resize to the minimum window size, and confirm all
   files can still be reached.
3. Add/rename/remove a file externally and Refresh. Toggle Subfolders, restart,
   and verify the choice persists. Cancel Change folder and confirm no change.
4. Open a valid card, return with Escape, then open a malformed or deleted MIDI
   file. The error must stay in the gallery without replacing the playable song.
5. Temporarily disconnect/rename the root folder, refresh/restart, then restore
   it and Refresh. The saved root must be retained and the gallery recover.
6. Open a standalone file, check Tracks/Play/FreePlay, and verify theme/settings
   controls and existing playback/practice behavior remain available.
