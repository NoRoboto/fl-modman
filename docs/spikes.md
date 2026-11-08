# Spikes (phase 0)

| Spike | Question | Result | Date |
|---|---|---|---|
| 0(b) | Does Sider read an LF-only `sider.ini` under Wine? | **Yes.** LAB launched with LF ini: 5 cpk.root + 18 lua modules loaded, commented camera modules skipped. CRLF restored after. Bonus: `fl27-lab.sh` launcher boots the LAB. | 2026-11-02 |
| 0(c) | Can Wine/Proton write through fuse-overlayfs (installer capture)? | **Yes.** umu/Proton 10 wrote into `Data/` and the root of an overlay over `~/Games/FL27`; files landed only in `upper/`, base untouched. | 2026-11-02 |
| 0(d) | Is the Ally `/home` reflink-capable? | **No**, ext4. Deploy order on the Ally: hardlink (assets) then copy. | 2026-11-02 |
| 0(a) | Can an egui/eframe window be driven by gamepad only on the Ally in Gaming Mode? | **Yes.** Fullscreen under gamescope; gilrs sees D-pad, A/B and, after adding axis handling with a 0.5 dead zone and auto-repeat, the left stick. Look is basic but readable. **egui approved** (Slint stays plan B). | 2026-11-02 |

**Phase 0 closed: all four spikes answered.**
