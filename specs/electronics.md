# Sigma — Electronics & ECU

*Companion to `sigma-spec.md`. The electronic half of Sigma (the Σ's second domain): a custom STM32 / Rust engine controller (§1–7) and an i.MX 8M Plus digital cockpit (§8), wrapped around the harvested Yamaha CP3. Chassis, mechanical, sourcing and finish rules live in `sigma-spec.md`; emissions/homologation in `emissions certification.md`.*

*Custom STM32 / Rust engine management for the Yamaha CP3 triple.*

## 1 · ECU

| Item | Spec |
|---|---|
| Core | Custom — STM32, Rust (Embassy async + real-time timer core) |
| Strategy | Speed-density, sequential injection (3-cyl), closed-loop lambda, ride-by-wire throttle |
| Replaces | Donor Yamaha ECU + harness (bespoke loom); must replicate or cleanly delete the immobiliser / CAN handshake to let the engine run |

## 2 · Sensor Stack

On a harvested CP3 you **interface to the engine's factory sensors** (characterize each on the bench — signal type, voltage range, connector) rather than re-sourcing generic parts. The only addition for closed-loop is a wideband.

| Sensor | Function | Source / notes |
|---|---|---|
| Crank position | Timing reference (absolute truth) | CP3 factory sensor + crank reluctor — characterize pattern + signal type on the engine (see §5) |
| Cam position | Cylinder phase (3-cyl sequential) | CP3 factory cam sensor |
| Throttle position ×2 | Ride-by-wire feedback | Dual TPS on the RbW throttle body — redundant, safety-critical |
| Grip / accelerator ×2 | Rider demand | Dual APP sensors at the twistgrip — redundant |
| MAP | Primary load (speed-density) | **Use the CP3 factory intake-pressure sensor**; confirm its type (MAP vs. MAF) on the engine. NA engine → a ~1-bar sensor is correct. Only add an aftermarket MAP if the factory part can't be read, and size it **~1-bar (not 3-bar boost-grade)** — a 3-bar wastes two-thirds of its range on an NA triple |
| CLT | Warmup / thermal comp | CP3 factory coolant-temp |
| IAT | Air-density correction | CP3 factory intake-air-temp |
| Wideband O₂ | Closed-loop fuel + diag | **Add** LSU 4.9 + CJ125 controller — single sensor post-collector for the triple. *(The one sensor with no Yamaha equivalent: the factory narrowband can't support tuning, so this stays Bosch.)* |
| Knock | Detonation / ign safety | CP3 factory knock sensor (if fitted) — else add piezo |
| Battery voltage | Injector/coil dead-time comp | STM32 ADC: divider + RC + TVS |

**Minimum required set:** crank, cam, dual TPS + dual APP (ride-by-wire), MAP, CLT, IAT, LSU 4.9 wideband, battery-voltage sensing. Optional: baro.

Characterize every factory CP3 sensor's type, range and connector on the bench — the ECU adapts to the engine's sensors; you don't re-source them. Add only the wideband (LSU 4.9 + CJ125).

## 3 · Connector Architecture

| Role | Connector | Notes |
|---|---|---|
| Main ECU harness | AMPSEAL 16 (23 or 35-pin) | IP67, compact, vibration-resistant; 14–16 A/terminal |
| Engine-bay sensors/actuators | Deutsch DT / DTM | Rugged, motorsport-proven; crank/cam, coils, injectors, subsystems |
| Injectors / throttle body | Yamaha CP3 factory connectors | Adapt the loom to the engine's OEM connectors (injectors, RbW throttle body, sensors) rather than re-pinning the engine |
| Mid-density OEM-style alt | Molex MX150 / MX23A | Compact sealed; tooling less universal |
| Internal PCB-to-harness | JST micro | Low-current signal only — not engine-bay exposure |

## 4 · ECU Design Principle

- Crank / cam = hardware truth (timer input-capture only)
- MAP / TPS = real-time load model
- Throttle = ride-by-wire: dual-redundant rider demand (APP) + dual throttle position, torque-based closed-loop, with an independent safety monitor and a fail-safe (spring-to-closed / fuel-cut) — safety-critical, kept off the slow loop
- Lambda = correction feedback (not timing-critical)
- Temp sensors = slow state-model inputs
- Connectors are part of the signal-integrity system — sensor noise, CAN errors and instability often trace to marginal pins, not firmware.

## 5 · CP3 Trigger & Sync

- **Crank:** the CP3 runs a crank-mounted reluctor read by the factory position sensor. **Read the exact tooth/missing-tooth pattern off the engine and Yamaha service data — do not assume or invent it.** Decode it as found.
- **Crank sensor:** characterize the CP3's factory sensor first (VR vs Hall) — a modern engine may already give a usable signal. Keep the **Hall-preferred** philosophy: if replacing, a gear-tooth Hall (ferrous-target, integrated bias magnet) gives a clean RPM-independent logic-level signal direct to the MCU. Confirm sensor type, air gap and supply on the actual engine.
- **Cam:** factory cam sensor → phase sync for 3-cylinder sequential fuel + ignition.
- **Firing:** even-firing inline-3, **240° between cylinders** → three evenly-spaced injection + ignition events per cycle; set the per-cylinder angles in the sequential scheduler.
- **TDC offset:** measure gap-to-TDC#1 on the real engine.
- Source exact figures from Yamaha service data + measure the donor; the CP3 has a large swap/standalone community for cross-checking.

## 6 · MCU Requirements (STM32)

| Need | Requirement |
|---|---|
| Crank / cam | ≥2 timer input-capture channels + free-running µs counter; hardware input filter (ISR-domain truth) |
| Crank/cam front-end | Characterize the CP3 sensor; Hall feeds logic-level direct (no conditioner) with a regulated 3-wire supply + input protection. VR conditioner (MAX9924-class) only if the factory sensor is VR and not replaced |
| Ride-by-wire | H-bridge / DC-motor driver for the throttle body + dual TPS & dual APP ADC inputs; closed-loop position control with an **independent safety monitor** and fail-safe (spring-to-closed / fuel-cut). Safety-critical |
| Analog | ≥8 ADC channels, DMA-fed (dual TPS, dual APP, MAP, CLT, IAT, Vbat, knock); faster ADC if knock-by-ADC |
| Injection | **3** hardware-timed compare outputs → low-side injector drivers (one per cylinder) |
| Ignition | **3** hardware-timed compare outputs → drive the **CP3 factory coil-on-plug** units, dwell-controlled (one per cylinder) |
| Comms | 1× CAN/FDCAN (dash/tuning, possible donor handshake), USB or UART (tuning/log), 1–2× SPI (CJ125 lambda, knock IC, ext flash) |
| Safety | Independent watchdog (IWDG) + brown-out detect; protected / level-shifted 5 V sensor inputs; throttle safety monitor |
| Aux PWM/GPIO | Fuel pump, fan, tacho, MIL (idle is handled by the ride-by-wire body) |

**Family:** the ride-by-wire motor control nudges this toward **STM32G474** (motor-control-grade — fast ADCs, timer/PWM for the throttle H-bridge, CORDIC, FDCAN), now sized for a 3-cylinder + RbW controller. **STM32F405/F407** remains a proven baseline (168 MHz, FPU, 14 timers, 3× ADC, 2× CAN — rusEFI's original target). Headroom: **STM32H743 / F767** (more RAM/flash, FDCAN, faster ADC) for on-board knock DSP + high-rate logging. All Embassy-supported.

## 7 · Open Electronics Items `[PENDING]`

- **Emissions control** (for Euro 5+ homologation) — closed-loop stoichiometric tuning (ECU already does this), **catalytic converter** in the collector, **evap/charcoal canister** + purge valve. The CP3 is a Euro 5 engine, so the combustion is already there; tune to the limits, then through the approval/inspection path per market (see `emissions certification.md`).
- **Immobiliser / CAN handshake** — confirm what the donor CP3 needs to start and run; the custom ECU must replicate or cleanly delete it.
- **Instruments** — **resolved: full digital cockpit on NXP i.MX 8M Plus — see §8.** (Not the Yamaha TFT: proprietary CAN, undrivable by the custom ECU.)
- **Electrical & lighting**
- **Charging** — use the CP3's factory stator + reg/rec; size for the new loads (ECU, ride-by-wire, lighting, fan)

---

## 8 · Cockpit / Digital Instrument Cluster

*Full digital cockpit on an NXP **i.MX 8M Plus** applications processor — a cockpit, not a gauge cluster: headroom for navigation, cameras and connectivity, at the mature/pragmatic tier just below the i.MX 95. Replaces the earlier Motoscope idea. Not the Yamaha TFT (proprietary CAN, undrivable by the custom ECU).*

**Why i.MX 8M Plus.** Nav/maps + cameras + connectivity need a Linux applications processor with a GPU, ISP and networking — beyond any STM32/MCU (including the NeoChrom H7S78: no ISP, no maps-class GPU, no native Linux stack). The 8M Plus is the mature sweet spot: **quad 64-bit Cortex-A53 @ 1.8 GHz + an 800 MHz Cortex-M7** (keeps the two-domain safety split), a Vivante GC7000UL 3D GPU (real map/graphics rendering), **dual ISP + two MIPI-CSI cameras**, a 2.3-TOPS NPU (bonus vision headroom), H.265 encode/decode, and — on the common SoMs — Wi-Fi 6 / BT 5.4 and **dual CAN-FD**. Shipping since ~2021, so cheaper, better-documented and lower-risk than the brand-new i.MX 95, on the same NXP Linux BSP family.

**Up / down alternatives.** **i.MX 95** if you later need the newest silicon, the Mali GPU, or *formal* ASIL-B functional safety — the 8M Plus has the M7 real-time core but is **not** ASIL-B-certified — at higher cost and freshest-BSP risk. **i.MX 6** only as a scope-cut floor: a proven, cheap, cluster-only chip (Cortex-A9, no safety M-core, weak for maps, no modern connectivity) — pick it only if the nav/infotainment ambition is dropped.

**Two-domain architecture (the cockpit split).**

| Domain | Cores | Role |
|---|---|---|
| **Safety / real-time** | Cortex-M7 @ 800 MHz (real-time; *not* ASIL-B-certified) | Instant-on telltales (ABS, oil, high-beam, turn, warnings) lit within ~1–2 s of power-on; real-time **CAN-FD** link to the custom ECU; runs Zephyr/bare-metal — never waits for Linux |
| **Rich / apps** | quad Cortex-A53 @ 1.8 GHz (Linux) | Navigation/maps, camera feeds, connectivity, infotainment HMI, GPU-rendered graphics |

The two domains are isolated: a maps/app crash on the A53 side cannot take down the M7 telltale/CAN cluster.

**Board strategy — prototype on EVK, productionise on SoM.**

| Item | Spec | Status |
|---|---|---|
| Prototype board | NXP **i.MX 8M Plus EVK** (8MPLUSLPD4-EVK) — full BSP + display/camera/connectivity bring-up. Or start on a SoM-vendor dev kit (below) that carries straight to the carrier. Verify part number at order | `[BUY]` eval |
| Production compute | **SoM on a bespoke carrier** — Variscite VAR-SOM-MX8M-PLUS / DART-MX8M-PLUS, Toradex Verdin iMX8M Plus, ADLINK LEC-IMX8MP, Forlinx FET-MX8MP-C, SolidRun, Engicam. Mature, huge choice; pick one with certified Wi-Fi 6 / BT for the cockpit. **Do not run the bare EVK in the bike.** | `[BUY]` + `[BESPOKE]` carrier |
| Power front-end | Bespoke automotive DC-DC off the 12–14 V loom — load-dump + transient + reverse protection. The EVK/SoM USB-C input will **not** survive a bike's electrical system. Size for SoM + display + cameras + modem | `[BESPOKE]` |

**Peripherals.**

| Item | Spec | Notes |
|---|---|---|
| Display | Wide **MIPI-DSI / LVDS TFT**, 1280×400-class "bar" cockpit format | Modern cockpit look; sunlight-readable + automotive-temp |
| Cameras | **2× MIPI-CSI** via the dual ISP (rear + blind-spot) | The 8M Plus has two ISPs / two camera inputs — good for two cameras; more than two needs muxing or extra hardware |
| Connectivity | **Wi-Fi 6 / BT 5.x** (integrated on most 8M Plus SoMs); **cellular M.2 modem + GNSS** for live nav | |
| Vehicle bus | **Dual CAN-FD** (native on the 8M Plus) to the custom ECU | Feeds the M7 safety cluster |

**Software.** A full embedded-Linux program on the A53 domain: **Yocto BSP** (or **Android Automotive OS**, NXP-supported, for a phone-like UX) hosting the maps engine, camera pipelines, connectivity stacks and HMI framework — **Qt**, or **Slint** to keep some Rust coherence (Slint runs on both i.MX Linux and MCUs). Separate **Zephyr** safety firmware on the M7 for telltales + CAN, plus boot orchestration between the two domains.

**Reality check.** This converts "instrument cluster" into "cockpit + infotainment" — a second serious embedded system alongside the Rust ECU, and a full Linux program (BSP + maps + cameras + connectivity), not a weekend cluster. Two things to nail early: camera count/resolution vs the ISP ceiling, and the automotive power/EMC design for a Linux computer hung off a motorcycle loom.

---

# Caveats (electronics)

1. **Custom ECU is a real engineering project** — trigger decoding, **ride-by-wire** (safety-critical electronic throttle), immobiliser/CAN handshake, fail-safes and bench/dyno tuning must all be proven before road use; the engine is unrideable until firmware and the sensor stack are validated.
2. **Modern engine, custom brain — that's the hard part** — the CP3 is new and Euro 5 (good for reliability, parts and emissions), but it's ride-by-wire + CAN + immobiliser. Running it standalone means re-creating throttle control, handshakes and emissions yourself, and buying a whole new donor bike to get the engine. Prototype on a used CP3 first.
3. **Characterize the CP3, don't assume it** — read the trigger reluctor pattern, sensor types/ranges and connectors off the actual engine and Yamaha service data; don't carry figures across from any other engine.
4. **The cockpit is a second whole project** — the i.MX 8M Plus digital cockpit (§8) is a full embedded-Linux effort (Yocto/Android Automotive BSP, maps, camera pipelines, connectivity) plus M7 safety firmware, running *alongside* the Rust ECU. Nav + cameras + connectivity turn a gauge cluster into an infotainment system; scope and staff it as its own build, not an afterthought to the engine ECU.
