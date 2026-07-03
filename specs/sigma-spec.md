# Sigma — Café Racer Build Spec Sheet

*Flat-black custom café racer · Yamaha CP3 triple · featherbed special*
*(Working title "Sigma": the Σ is mechanical + electronic summed into one machine.)*

Two domains summed into one machine: a bespoke chromoly featherbed chassis + mechanicals (this doc) wrapped around a new Yamaha CP3 inline-triple, plus a ground-up STM32 / Rust engine controller and i.MX 95 digital cockpit (**`electronics.md`**).

- **Strategy:** speed-density (MAP-based), sequential injection, closed-loop lambda, ride-by-wire throttle
- **Markets:** Mexico, Canada, EU, UK *(US dropped as a road target — see Emissions & homologation; would be display-only)*
- **Sourcing rule:** components from Canada, Mexico, or EU *(engine is a new Yamaha — Japanese marque, bought new via an EU/CA/MX dealer)*
- **Yamaha-first for powertrain ancillaries:** the donor is bought whole and harvested, so its Yamaha parts (ignition coils, injectors, assist/slipper clutch, in-tank fuel pump/regulator, stator + reg/rec, radiator + fan, all factory sensors) are already paid for and CP3-matched — use them, not catalog substitutes. Premium chassis items (Öhlins, Brembo, Kineo) stay as specced; they're the build's purpose, not ancillaries to economize.
- **Finish:** flat black throughout — per-material finish (anodize, powder, cerakote) to suit each component
- **Status tags:** `[LOCKED]` · `[BESPOKE]` one-off fabrication (in-house or commissioned) · `[DONOR]` buy used · `[BUY]` catalog · `[PENDING]`

> **Emissions & homologation.** Target = a current-standard (Euro 5+) build, not a donor-cert hand-me-down. One technical spec (closed-loop + cat + evap) covers all four markets; the paperwork differs — EU/UK yield a certificate, Canada/Mexico give registration + inspection, US is dropped (display-only). **Full breakdown, per-market paths and hardware checklist in `emissions certification.md`.**

---

# Part I · Chassis & Mechanical

## 1 · Powertrain (mechanical)

| Item | Spec | Status |
|---|---|---|
| Engine | Yamaha CP3 — 890 cc liquid-cooled inline-3, DOHC 12v, crossplane crank (even firing, 240° spacing) | `[LOCKED]` |
| Output | ~117 hp (claimed) @ ~10,000 rpm, ~93 Nm; ~10,500 rpm redline | — |
| Induction | EFI, ride-by-wire (YCC-T) | — |
| Cooling / drive | Liquid-cooled · chain final | — |
| Emissions base | Euro 5 from the factory — a head start on re-homologation (see `emissions certification.md`) | — |
| Sourcing | **New** — harvested from a current Yamaha donor: engine + gearbox + ride-by-wire throttle bodies + factory sensors | `[BUY]` new donor |

**CP3 donor models (identical engine — buy the cheapest to harvest):**

- Yamaha MT-09 — cheapest CP3 donor; first choice for a harvest
- Yamaha XSR900 — café/retro variant (same engine + frame as MT-09)
- Yamaha Tracer 9 / YZF-R9 — same 890 cc CP3, touring / sportbike tunes

**Buy notes:**

- The custom ECU (see **`electronics.md`**) replaces the donor's ECU and loom — but you **keep** the engine, gearbox, ride-by-wire throttle bodies, factory sensors, **ignition coils, injectors, the assist/slipper clutch, the in-tank fuel pump + regulator, the stator + reg/rec, and the radiator + fan**; the ECU is built to interface with all of it. Default to the donor's Yamaha parts for powertrain ancillaries — already bought with the engine and matched to the CP3.
- The CP3 is **ride-by-wire**: the throttle body's motor + dual throttle-position and dual grip sensors must be driven and read by the custom ECU — a safety-critical subsystem (see `electronics.md` §6).
- The donor may need an **immobiliser / CAN handshake** to run; the custom ECU must replicate or cleanly delete it.
- Cheapest path to a new engine is a new **MT-09** — harvest engine + throttle bodies + sensors + wiring reference; the rest of the bike is surplus.
- **Prototyping:** develop and bench the ECU on a cheap **used CP3** (used MT-09) before committing the new engine — same trigger, sensors and ride-by-wire as the build engine, so the work carries straight across.

## 2 · Chassis

| Item | Spec | Status |
|---|---|---|
| Builder | In-house fabrication | — |
| Frame | Bespoke featherbed-style chromoly steel, built around the CP3 + radiator (narrow inline-3 packages tight; front-mount radiator). Modern featherbed special — Triton-lineage triple | `[BESPOKE]` |
| Rear shock | **Öhlins STX 46 Blackline** — 46 mm high-pressure gas **monotube**, black body + spring. Order the **piggyback + compression-adjustable** configuration with the **hydraulic preload** option to get full adjustability (compression + rebound + preload) — *not all STX 46 variants carry the compression adjuster, so specify it explicitly.* Built to order: eye-to-eye length + spring rate derived from the **frozen** rising-rate linkage motion ratio & rider weight. ⚠ Ride height: length-adjustable lower eyelet is a TTX-tier feature and unconfirmed on the STX 46 — either option it, or set ride height via the built-to-order eye-to-eye length + linkage (the 170/60-17 rear's ~6 mm rise is baked in at order time). Matched to the FG 621 front | `[LOCKED]` / `[BUY]` built-to-order |
| Monoshock linkage | Bespoke rising-rate linkage to the Öhlins shock | `[BESPOKE]` |
| Swingarm | Bespoke fabricated, chain drive, matched to the linkage, sized for the **170 rear** (was 160 — see tire note). Check tire-to-swingarm/chain clearance at the 170 section width | `[BESPOKE]` |
| Rear wheel | Kineo true-tubeless spoked — 17 × 4.5", matched to the front; includes 8-pin cush drive + Ergal sprocket carrier. Matte black (Italy). 4.5" rim suits the 170/60-17 | `[BUY]` made-to-order |
| Rear tire | **170/60-17** — Pirelli Scorpion Rally STR (72V). *(Was 160/60-17; STR is not made in 160/60-17, so moved to 170/60-17, an available STR size that suits the 4.5" rim — see tire note.)* | `[LOCKED]` |

**Tire note (Pirelli Scorpion Rally STR — front + rear).** Decision: run the road-biased rally tire for a scrambler-flavoured stance that still rides honestly on tarmac. Front **120/70-17 (58H)**, rear **170/60-17 (72V)** — a matched pair, both genuine STR sizes.

- **Why 170, not 160.** The STR is an ADV tire and Pirelli does not catalogue it in 160/60-17 (the original locked rear). The available 17" STR rears are 150/60, 150/70, 170/60, 180/55; 170/60-17 is the best fit for the 17 × 4.5" Kineo rim. So the rear lock moved 160 → 170.
- **Geometry knock-on.** 170/60-17 is ~12 mm larger in overall diameter than 160/60-17 → ~6 mm more rear ride height at the axle, plus a small final-drive/speedo change. Absorbed at **shock-order time** via the built-to-order eye-to-eye length + the bespoke rising-rate linkage (the STX 46's on-the-fly ride-height adjuster is unconfirmed — see rear-shock note). Set ride height/trail **on the STR**, not on a sport tire: this is a tall, round ADV profile and steers slower than sport rubber, so the chassis must be validated on the actual tire.
- **Swingarm.** Was "sized for the 160 rear" → now 170; confirm tire-to-arm and tire-to-chain clearance at the wider section.
- **Front speed rating — homologation check.** The STR front 120/70-17 is rated **58H (210 km/h)**. A standalone CP3 runs near that, and EU/UK approval requires the fitted tire's speed rating to meet or exceed the certified top speed. Confirm the 58H front is acceptable (or restrict/declare top speed accordingly) before treating the front size as final. Rear 72V (240 km/h) is clear. *(Logged in `emissions certification.md`.)*

## 3 · Front End

| Item | Spec | Status |
|---|---|---|
| Fork | Öhlins FG 621 — 43 mm conventional (RWU, non-inverted), black. Largest non-inverted Öhlins: 800 mm length, 120 mm stroke, NIX 30 cartridge, 9.5 N/mm springs, 32 mm axle, fully adjustable. Ships universal (no yokes, no caliper/fender mounts). Source: Zodiac (NL, ships MX) / EU dealer, ~$2,300–2,800 | `[LOCKED]` / `[BUY]` |
| Yokes | Bespoke (in-house) — cut to FG 621 43 mm tubes + featherbed neck + café-trail offset | `[BESPOKE]` |
| Steering stem | Bespoke billet steel (4140 chromoly), heat-treated — not aluminium. Safety-critical: validate for steering loads | `[BESPOKE]` |
| Headstock bearing | **Yamaha CP3-family tapered set — upper 25×47×15, lower 30×55×17** (+ seals). The asymmetric pair the MT-09/XSR900/XSR900 GP run — *not* a symmetric 30×55×17 both-ends set. Design the bespoke stem journals + frame-neck bore to these two sizes. Tapered roller = rigid, preload-adjustable. ⚠ Confirm the exact set on a **gen-2 (’21+) donor / current Yamaha microfiche** — the 2021 frame was redesigned; upper 25×47×15 / lower 30×55×17 is confirmed on gen-1 (’14–’18) and is the family norm, but verify it carries to the GP before cutting journals. | `[LOCKED]` |
| Stem geometry reference | **Yamaha XSR900 GP** (gen-2 CP3 platform — MT-09/XSR900 ’21+ frame). Reference only; same-engine, same-power-class steering head, so a better-matched reference than a ~200 hp superbike stem. Final stem dims from a **measured donor stem + Yamaha microfiche**, never invented | `[LOCKED]` |
| Bearing kit | All Balls **22-2004** — MT-09/XSR900 set (upper 25×47×15 + lower 30×55×17, top+bottom+seals). The asymmetric set is the Yamaha middleweight norm (and shared with e.g. Suzuki Katana 600/750 ’89+), so sourcing stays trivial. Verify the gen-2/GP part number against a current catalog. | `[BUY]` |

**Offset / trail.** Set at the yoke (bespoke). Final trail = featherbed neck rake + yoke offset — independent of the Yamaha stem (the stem only sets the bearing interface). Stem dimensions come from a measured XSR900 GP / CP3-family donor stem + Yamaha microfiche — never invented. See the stem spec sheet (`steering-stem-spec.html`).

**Rigidity note.** Switching to the Yamaha set drops the *upper* bearing from 30×55×17 (the old symmetric reference) to 25×47×15 — a smaller upper journal. For ~130 hp this is not the limiting element: neck-tube section and stem diameter dominate steering-head stiffness, both bespoke, and this is the exact set Yamaha runs behind the same CP3 engine. Keep a larger-taper upper on the table only if a neck-stiffness check calls for it; it buys marginal rigidity at the cost of a bigger, heavier neck and the loss of the Yamaha set's drop-in availability.

### Front wheel & brake

| Item | Spec | Status |
|---|---|---|
| Front wheel | Kineo true-tubeless spoked — 17 × 3.5", 28-spoke, forged 7000-alloy hub + CNC billet rim, matte black. TÜV-certified. Made to order around the 32 mm axle + chosen discs (Italy). | `[BUY]` made-to-order |
| Front tire | **120/70-17 (58H)** — Pirelli Scorpion Rally STR. ⚠ 58H = 210 km/h; verify vs certified top speed for EU/UK approval (see tire note + `emissions certification.md`) | `[LOCKED]` |
| Front axle | **32 mm — use the FG 621's supplied axle.** The fork is built around a 32 mm wheel axle and ships with axle sleeve/spacer hardware; the made-to-order Kineo wheel is built with bearings/spacers to suit the 32 mm axle. ⚠ Confirm exact box contents (axle + how many spacer sets) with Zodiac/Öhlins — a bespoke spindle is most likely *not* needed | `[BUY]` / confirm contents |
| Front discs | Twin 320 mm floating — Brembo; pick any stocked 320 mm floating pattern, Kineo builds the carriers to the chosen disc | `[BUY]` |
| Calipers | Brembo M4 (M4.34) cast monobloc, 4 × 34 mm, **100 mm radial mount** (Euro pitch), black. KBA homologation docs available. | `[BUY]` |
| Caliper brackets | Bespoke billet radial brackets — 100 mm caliper pitch, set to the 320 mm disc radius; the bracket replaces Brembo's offset adapters | `[BESPOKE]` |
| Master cylinder | Brembo 19 RCS radial (Brembo's recommended match for the 34 mm-piston M4) | `[BUY]` |
| Brake lines | Braided stainless — Spiegler (DE) / Goodridge | `[BUY]` |
| Front fender | Bespoke stays off the FG 621 lowers; carbon or flat-black alloy | `[BESPOKE]` |

**Build order (everything keys off the disc).** Pick the 320 mm disc first — its PCD/offset is the datum. Kineo then builds the hub, bearings, spacers and disc carriers to that disc + the 32 mm axle; the axle is fabricated in-house to the fork feet; the bespoke radial bracket places the M4 (100 mm pitch) at the 320 mm friction radius.

**Verify with the fork in hand — and check what the Zodiac package includes.** The bare Öhlins FG 621 lower has no caliper provision, *but* Zodiac's FG620/621 package (this spec's fork source) reportedly ships fork legs with axle clamps that already have **caliper mounting points**. Confirm directly with Zodiac which version carries them and what pattern — likely Harley-oriented, so it may need adaptation for the M4's 100 mm radial pitch. If the Zodiac mounts suit or adapt, the caliper problem is largely solved at purchase. **Decided: no machining of the fork.** If mounts aren't included, use a bespoke **axle-datum / foot-clamp** caliper bracket (Öhlins to sanction clamp location — not machined pads), with Rebuffini's FG620/621 brackets (Italy, EU) as reference/base. Also order the Kineo wheel early — lead time runs up to ~18 weeks.

## 4 · Sourcing Summary (mechanical)

| Buy now (catalog / new) | Buy used (prototyping) | Bespoke (one-off fabrication) |
|---|---|---|
| New CP3 donor (MT-09) — engine + box + RbW + sensors | Used CP3 (used MT-09) — ECU bench/proto mule | Featherbed frame (in-house) |
| Öhlins FG 621 fork (black) — Zodiac/EU | | Swingarm + monoshock linkage |
| Öhlins STX 46 Blackline rear shock | | Triple yokes (43 mm) |
| All Balls 22-2004 bearing kit (Yamaha set) | | Billet-steel steering stem |
| Kineo front + rear wheels (made-to-order) | | Engine build/display cradle |
| Brembo M4 calipers + 320 mm discs | | Front axle (32 mm) + caliper brackets + fender stays |
| Brembo 19 RCS master + braided lines | | |

## 5 · Open Decisions (mechanical) `[PENDING]`

- **Bodywork** — tank, seat/cowl, café cockpit (clip-ons, rearsets)
- **Exhaust** — must be a **catalysed** system (cat in the collector) to hit the Euro 5+ target. *(Clutch resolved: retain the CP3's factory assist/slipper clutch — it comes with the engine/gearbox. Primary drive is internal to the CP3.)*

---

# Part II · Electronics & ECU → moved

*The electronic half of Sigma — the custom STM32 / Rust engine controller and the i.MX 95 digital cockpit — now lives in its own companion: **`electronics.md`**.*

---

# Caveats (chassis & homologation)

1. **Steering stem is safety-critical** — billet steel + heat-treat, dimensions from a measured XSR900 GP / CP3-family donor stem + Yamaha microfiche, validated in-house. No invented numbers.
2. **Emissions is a homologation project** — current-standard target (Euro 5+) via cat + closed-loop + evap; the CP3's factory Euro 5 base is a head start, but a custom ECU + custom exhaust means you re-homologate regardless. EU/UK give a real certificate, Canada/Mexico give registration + inspection, US is display-only and dropped. Full detail in `emissions certification.md`.
3. **Bespoke = cost + lead time** — frame, swingarm, yokes, stem and cradle are one-off-grade, not catalog parts.

*(Electronics caveats — custom ECU, standalone brain, CP3 characterization, cockpit — now live in `electronics.md`.)*
