# Test fixtures

Everything here is synthetic. No file comes from the game, SmokePatch, Sider or a published mod.

| Folder | Contents |
|---|---|
| `fl27-game/` | A fake FL27 install. Every file is a short text placeholder (for example, `FL_2027.exe` holds the text `fake FL_2027.exe image`). `sider.ini` and `map_teams.txt` are hand-written files that copy only the layout quirks the parsers must keep: CRLF, mixed tab/space indentation, a duplicated key, trailing comments. |
| `fl26/` | Hand-written `sider.ini` and `map_teams.txt` with the same quirks, used by the codec round-trip tests. |
| `mods/` | Four small mod packages. Their payloads are text placeholders with game-like file names. |

`.gitattributes` marks this tree `-text` so the CRLF line endings survive checkout.
