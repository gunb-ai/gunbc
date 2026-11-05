import SwiftUI

/// The garden along the bottom of every screen: pixel flowers on a strip of grass, and gunbc workers
/// walking between them and stopping to tend one. Decoration only; it takes no touches.
///
/// THE WORKER is the daily workspace's build-meter worker (gunbc.site.element.build_meter): the gunbai
/// robot mark at strip size, 20 units wide, transcribed here box by box from the values that module
/// derives (body insets from the squircle corner, antenna, light, shade, feet, eyes, mouth), in its
/// palette (--meter-worker, --meter-worker-light, --meter-worker-shade, --meter-worker-feature).
/// The flowers are new: no flower art exists in the repository or on the served workspace.
struct GardenStrip: View {
    /// Bots approved this session, who join the garden with their antenna lit green.
    var recruits = 0
    @Environment(\.colorScheme) private var scheme
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.displayScale) private var displayScale

    static let height: CGFloat = 56

    var body: some View {
        TimelineView(.animation(minimumInterval: 1.0 / 12, paused: reduceMotion)) { timeline in
            Canvas { ctx, size in
                let t = reduceMotion ? 0 : timeline.date.timeIntervalSinceReferenceDate
                Garden.draw(ctx: ctx, size: size, t: t, palette: Garden.Palette(scheme), scale: displayScale, recruits: recruits)
            }
        }
        .frame(height: Self.height)
        // The grass continues under the home indicator rather than stopping above it.
        .background(alignment: .bottom) {
            Garden.Palette(scheme).grass.frame(height: 200).offset(y: 200 - Garden.grassHeight)
        }
        .accessibilityHidden(true)
        .allowsHitTesting(false)
    }
}

enum Garden {
    // ── Palette ──────────────────────────────────────────────────────────────────────────────
    struct Palette {
        let worker, light, shade, feature, grass, grassLight, stem, leaf: Color
        init(_ scheme: ColorScheme) {
            // The workspace's two worker palettes (light page, dark page).
            if scheme == .dark {
                worker = Color(hex: 0xCEC5B6); light = Color(hex: 0xE8E1D6); shade = Color(hex: 0x9C9282); feature = Color(hex: 0x151514)
                grass = Color(hex: 0x3A4535); grassLight = Color(hex: 0x4A5644); stem = Color(hex: 0x56634F); leaf = Color(hex: 0x66735D)
            } else {
                worker = Color(hex: 0x0F0F0F); light = Color(hex: 0x3D3C39); shade = Color(hex: 0x000000); feature = Color(hex: 0xF4F1EC)
                grass = Color(hex: 0x7F8C70); grassLight = Color(hex: 0x95A186); stem = Color(hex: 0x66735A); leaf = Color(hex: 0x828F74)
            }
        }
    }

    struct Box { let x, y, w, h: Int }

    // ── The worker (gunbc.site.element.build_meter), 20 × 23 units, y from -5 (antenna top) to 17 ──
    enum Worker {
        static let width = 20
        static let top = -5
        static let rows = 23
        /// build_meter_body_inset for y in 0..<16 (corner radius 6 on a 20 × 16 body).
        static let insets = [4, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 4]
        static let antenna = [Box(x: 9, y: -1, w: 2, h: 1), Box(x: 8, y: -2, w: 4, h: 1), Box(x: 7, y: -4, w: 6, h: 2), Box(x: 8, y: -5, w: 4, h: 1)]
        /// DIVERGES from build_meter_light_rows (a long arc over rows 1-4, which reads as a stripe at
        /// the booth's size): a two-pixel glint at the same corner.
        static let light = [Box(x: 4, y: 2, w: 2, h: 1), Box(x: 3, y: 3, w: 1, h: 1)]
        static let shade = [Box(x: 2, y: 14, w: 16, h: 1), Box(x: 4, y: 15, w: 12, h: 1), Box(x: 19, y: 6, w: 1, h: 6)]
        static let feet = [Box(x: 3, y: 16, w: 4, h: 2), Box(x: 13, y: 16, w: 4, h: 2)]
        static let eyes = [Box(x: 4, y: 4, w: 3, h: 3), Box(x: 13, y: 4, w: 3, h: 3)]
        static let mouth = Box(x: 8, y: 7, w: 4, h: 1)
    }

    // ── Flowers: one character per unit. P petal, C centre, S stem, L leaf. ─────────────────────
    struct Flower { let art: [String]; let petal: Color; let centre: Color }
    static let flowerArt: [[String]] = [
        [".P.P.", "PPCPP", ".PPP.", "..S..", "L.S..", ".LS.L", "..SL.", "..S.."],
        ["P.P.P", "PPPPP", "PPPPP", ".PPP.", "..S..", "LLS..", "..SLL", "..S.."],
        [".P.", "PCP", ".P.", ".S.", "LS.", ".SL", ".S."],
        // Wilted: the head hangs off the top of the stem.
        ["PP...", "PCS..", ".PS..", "..S..", "..SL.", ".LS..", "..S..", "..S.."],
    ]
    /// Dusty, faded petals: nothing here is in bloom so much as holding on.
    static let petals: [(Color, Color)] = [
        (Color(hex: 0xB89AA0), Color(hex: 0xC9B27A)),
        (Color(hex: 0x9C95AD), Color(hex: 0xC9B27A)),
        (Color(hex: 0xB7A274), Color(hex: 0x8C7A55)),
        (Color(hex: 0x8FA0AE), Color(hex: 0xD8D1BE)),
        (Color(hex: 0xA8A49A), Color(hex: 0xC9B27A)),
    ]

    /// Units are whole device pixels, so every box lands on the pixel grid and the art stays crisp.
    static func workerUnit(_ scale: CGFloat) -> CGFloat { (5 / 3 * scale).rounded() / scale }
    static func flowerUnit(_ scale: CGFloat) -> CGFloat { (4 * scale).rounded() / scale }
    static let grassHeight: CGFloat = 6
    static let speed: Double = 22        // points per second
    static let tend: Double = 2.6        // seconds spent at a flower
    static let workerCount = 3
    static let maxRecruits = 2
    static let success = Color(hex: 0x3FA34D)   // the workspace's antenna "success" paint
    static let stop = Color(hex: 0xD64545)      // and its "stop" paint

    /// Flower positions across the strip: one every ~46 pt, nudged deterministically so the row is
    /// not a fence. Index picks the art and the colour.
    static func flowers(width: CGFloat, pitch: CGFloat = 46) -> [(x: CGFloat, kind: Int, colour: Int)] {
        let n = max(Int(width / pitch), 1)
        let step = width / CGFloat(n)
        return (0..<n).map { i in
            let jitter = CGFloat((i * 37) % 17) - 8
            return (step * (CGFloat(i) + 0.5) + jitter, (i * 7 + 1) % flowerArt.count, (i * 3 + 2) % petals.count)
        }
    }

    // ── Drawing ──────────────────────────────────────────────────────────────────────────────
    static func draw(ctx: GraphicsContext, size: CGSize, t: Double, palette p: Palette, scale: CGFloat, recruits: Int = 0,
                     regulars: Int = workerCount, workerPoints: CGFloat? = nil, flowerPoints: CGFloat? = nil, pitch: CGFloat = 46) {
        let wu = workerPoints.map { ($0 * scale).rounded() / scale } ?? workerUnit(scale)
        let fu = flowerPoints.map { ($0 * scale).rounded() / scale } ?? flowerUnit(scale)
        let ground = size.height - grassHeight
        drawGrass(ctx: ctx, width: size.width, ground: ground, palette: p)

        let fl = flowers(width: size.width, pitch: pitch)
        for f in fl { drawFlower(ctx: ctx, art: flowerArt[f.kind], colours: petals[f.colour], palette: p, centreX: f.x, base: ground, unit: fu) }

        // Each worker keeps to its own stretch of the garden, so they never walk through each other.
        // Recruits (approved bots) take the stretches after the regulars'.
        let total = regulars + min(recruits, maxRecruits)
        let zone = size.width / CGFloat(total)
        for w in 0..<total {
            let lo = zone * CGFloat(w), hi = lo + zone
            var stops = fl.map { $0.x }.filter { $0 >= lo && $0 < hi }
            if stops.count < 2 { stops = [lo + zone * 0.3, lo + zone * 0.7] }
            let pose = workerPose(index: w, t: t, stops: stops, lo: lo, hi: hi, unit: wu)
            let recruit = w >= regulars
            drawWorker(ctx: ctx, palette: p, x: pose.x, base: ground, facingLeft: pose.facingLeft, lift: pose.lift, blink: pose.blink, unit: wu,
                       face: .tired, antenna: recruit ? success : nil)
            if pose.tending { drawSparkle(ctx: ctx, x: pose.flowerX, base: ground - 8 * fu - 10, phase: pose.phase) }
        }
    }

    static func drawGrass(ctx: GraphicsContext, width: CGFloat, ground: CGFloat, palette p: Palette) {
        ctx.fill(Path(CGRect(x: 0, y: ground, width: width, height: grassHeight)), with: .color(p.grass))
        var x: CGFloat = 0
        var i = 0
        while x < width {
            let h: CGFloat = i % 3 == 0 ? 4 : 2
            ctx.fill(Path(CGRect(x: x, y: ground - h, width: 2, height: h)), with: .color(p.grassLight))
            x += i % 2 == 0 ? 6 : 10
            i += 1
        }
    }

    static func drawFlower(ctx: GraphicsContext, art: [String], colours: (Color, Color), palette p: Palette, centreX: CGFloat, base: CGFloat, unit u: CGFloat) {
        let w = CGFloat(art[0].count) * u
        let left = (centreX - w / 2).rounded()
        let top = base - CGFloat(art.count) * u
        for (r, row) in art.enumerated() {
            for (c, ch) in row.enumerated() {
                let colour: Color
                switch ch {
                case "P": colour = colours.0
                case "C": colour = colours.1
                case "S": colour = p.stem
                case "L": colour = p.leaf
                default: continue
                }
                ctx.fill(Path(CGRect(x: left + CGFloat(c) * u, y: top + CGFloat(r) * u, width: u, height: u)), with: .color(colour))
            }
        }
    }

    struct Pose { let x: CGFloat; let facingLeft: Bool; let lift: Int; let blink: Bool; let tending: Bool; let flowerX: CGFloat; let phase: Double }

    /// Where worker `index` is at time t: it walks at a constant speed from flower to flower in a
    /// fixed pseudo-random order, and stands at each for `tend` seconds, tapping. Pure in t, so the
    /// strip needs no state and every redraw agrees with the last.
    static func workerPose(index: Int, t: Double, stops: [CGFloat], lo: CGFloat, hi: CGFloat, unit: CGFloat) -> Pose {
        // A fixed itinerary of 8 stops for this worker.
        var seed = UInt64(index + 1) &* 0x9E37_79B9_7F4A_7C15
        func next() -> Int { seed = seed &* 6364136223846793005 &+ 1442695040888963407; return Int(seed >> 33) }
        var route = [next() % stops.count]
        while route.count < 8 {
            let s = next() % stops.count
            if s != route.last! { route.append(s) }
        }
        // The worker stands just beside the flower it tends.
        let standOffset = CGFloat(Worker.width) * unit / 2 + 8
        func stand(_ s: Int, from: CGFloat) -> CGFloat {
            let fx = stops[s]
            let left = fx - standOffset, right = fx + standOffset
            let candidate = from <= fx ? left : right
            return min(max(candidate, lo + standOffset), hi - standOffset)
        }
        var legs: [(from: CGFloat, to: CGFloat, walk: Double, flower: CGFloat)] = []
        var at = stand(route[0], from: 0)
        for k in 1...route.count {
            let s = route[k % route.count]
            let to = stand(s, from: at)
            legs.append((at, to, Double(abs(to - at)) / speed, stops[s]))
            at = to
        }
        let period = legs.reduce(0) { $0 + $1.walk + tend }
        var local = (t + Double(index) * 7.3).truncatingRemainder(dividingBy: period)
        let blink = (t + Double(index) * 1.7).truncatingRemainder(dividingBy: 4.1) < 0.15
        for leg in legs {
            if local < leg.walk {
                let f = leg.walk == 0 ? 1 : local / leg.walk
                let x = leg.from + (leg.to - leg.from) * CGFloat(f)
                // Step-quantised like the workspace's steps(): two-frame bob.
                let frame = Int(local * 6) % 2
                return Pose(x: (x / unit).rounded() * unit, facingLeft: leg.to < leg.from, lift: frame, blink: blink, tending: false, flowerX: leg.flower, phase: 0)
            }
            local -= leg.walk
            if local < tend {
                let tap = Int(local * 3) % 2 == 0 ? 1 : 0
                return Pose(x: leg.to, facingLeft: leg.flower < leg.to, lift: tap, blink: blink, tending: true, flowerX: leg.flower, phase: local / tend)
            }
            local -= tend
        }
        return Pose(x: at, facingLeft: false, lift: 0, blink: false, tending: false, flowerX: at, phase: 0)
    }

    /// build_meter's expressions: BuildMeterDeadpan, BuildMeterContent, BuildMeterSad, and tired --
    /// the deadpan face under its drooping upper lid (build_meter_droop_class), the house default.
    /// lethargic, moody, angry and depressed are the mark studio's moods for a tired machine
    /// (gunbc.design.brand_mark_studio StudioExpression, branch session/swift-badger-498-studio): a lid
    /// over each square eye, at the pixel grain.
    enum Face { case deadpan, tired, content, sad, back, lethargic, moody, angry, depressed }

    static func drawWorker(ctx: GraphicsContext, palette p: Palette, x centreX: CGFloat, base: CGFloat, facingLeft: Bool, lift: Int, blink: Bool, unit u: CGFloat,
                           face: Face = .deadpan, antenna: Color? = nil, costume: Int? = nil) {
        let w = CGFloat(Worker.width) * u
        // Snap to the unit grid (itself whole device pixels), so rows meet without seams.
        let left = ((centreX - w / 2) / u).rounded() * u
        // The feet's bottom row (y = 17) stands on the grass; lift raises the body by whole units.
        let originY = ((base - CGFloat(18 + lift) * u) / u).rounded() * u
        func rect(_ b: Box, _ c: Color) {
            // Mirror within the 20-unit frame when facing left; the mark is symmetric, the shading is not.
            let bx = facingLeft ? Worker.width - b.x - b.w : b.x
            ctx.fill(Path(CGRect(x: left + CGFloat(bx) * u, y: originY + CGFloat(b.y) * u, width: CGFloat(b.w) * u, height: CGFloat(b.h) * u)), with: .color(c))
        }
        for (y, inset) in Worker.insets.enumerated() { rect(Box(x: inset, y: y, w: Worker.width - 2 * inset, h: 1), p.worker) }
        Worker.light.forEach { rect($0, p.light) }
        Worker.shade.forEach { rect($0, p.shade) }
        // Feet stay on the ground while the body bobs; on a lifted frame the leading foot steps up.
        for (k, f) in Worker.feet.enumerated() {
            let raised = lift == 1 && k == (facingLeft ? 0 : 1) ? 1 : 0
            rect(Box(x: f.x, y: f.y + lift - raised, w: f.w, h: f.h), p.shade)
        }
        let m = Worker.mouth
        switch face {
        case .deadpan:
            for e in Worker.eyes { rect(blink ? Box(x: e.x, y: e.y + e.h - 1, w: e.w, h: 1) : e, p.feature) }
            rect(m, p.feature)
        case .back:
            break                                   // seen from behind: no face, only the antenna
        case .tired:
            for e in Worker.eyes { rect(blink ? Box(x: e.x, y: e.y + e.h - 1, w: e.w, h: 1) : Box(x: e.x, y: e.y + 1, w: e.w, h: e.h - 1), p.feature) }
            rect(m, p.feature)
        case .content:
            // build_meter_arc_eye_rows and build_meter_bent_mouth_rows(up: true).
            for e in Worker.eyes {
                rect(Box(x: e.x + e.w / 2, y: e.y, w: 1, h: 1), p.feature)
                rect(Box(x: e.x, y: e.y + 1, w: 1, h: 1), p.feature)
                rect(Box(x: e.x + e.w - 1, y: e.y + 1, w: 1, h: 1), p.feature)
            }
            rect(Box(x: m.x, y: m.y - 1, w: 1, h: 1), p.feature)
            rect(Box(x: m.x + 1, y: m.y, w: m.w - 2, h: 1), p.feature)
            rect(Box(x: m.x + m.w - 1, y: m.y - 1, w: 1, h: 1), p.feature)
        case .sad:
            // Square eyes under the drooping upper lid, and build_meter_bent_mouth_rows(up: false).
            for e in Worker.eyes { rect(Box(x: e.x, y: e.y + 1, w: e.w, h: e.h - 1), p.feature) }
            rect(Box(x: m.x, y: m.y + 1, w: 1, h: 1), p.feature)
            rect(Box(x: m.x + 1, y: m.y, w: m.w - 2, h: 1), p.feature)
            rect(Box(x: m.x + m.w - 1, y: m.y + 1, w: 1, h: 1), p.feature)
        case .lethargic:
            // The lid down to the eye's last row.
            for e in Worker.eyes { rect(Box(x: e.x, y: e.y + e.h - 1, w: e.w, h: 1), p.feature) }
            rect(m, p.feature)
        case .moody:
            // Lidded, and lower at the outer corners.
            for (k, e) in Worker.eyes.enumerated() {
                rect(Box(x: e.x, y: e.y + 2, w: e.w, h: 1), p.feature)
                rect(Box(x: k == 0 ? e.x + 1 : e.x, y: e.y + 1, w: e.w - 1, h: 1), p.feature)
            }
            rect(Box(x: m.x + 1, y: m.y, w: m.w - 2, h: 1), p.feature)
        case .angry:
            // Lower at the inner corners.
            for (k, e) in Worker.eyes.enumerated() {
                rect(Box(x: e.x, y: e.y + 2, w: e.w, h: 1), p.feature)
                rect(Box(x: k == 0 ? e.x : e.x + 1, y: e.y + 1, w: e.w - 1, h: 1), p.feature)
            }
            rect(m, p.feature)
        case .depressed:
            for e in Worker.eyes { rect(Box(x: e.x, y: e.y + e.h - 1, w: e.w, h: 1), p.feature) }
            rect(Box(x: m.x, y: m.y + 1, w: 1, h: 1), p.feature)
            rect(Box(x: m.x + 1, y: m.y, w: m.w - 2, h: 1), p.feature)
            rect(Box(x: m.x + m.w - 1, y: m.y + 1, w: 1, h: 1), p.feature)
        }
        // What it is wearing, over the body and face, with the antenna up through it: its stem behind
        // the hat, its ball (lit with the outcome, build_meter_antenna_paint) clear above the top.
        let outfit = costume.map { Wardrobe.costumes[$0 % Wardrobe.costumes.count] }
        let antennaLift = outfit?.antennaLift ?? 0
        for k in 0...antennaLift { rect(Box(x: 9, y: -1 - k, w: 2, h: 1), p.worker) }
        for part in outfit?.parts ?? [] {
            if face == .back && part.front { continue }          // a tie or glasses is on the face side
            rect(part.box, part.paint == Wardrobe.body ? p.worker : (part.paint == Wardrobe.feature ? p.feature : Color(hex: part.paint)))
        }
        for (k, b) in Worker.antenna.enumerated() where k > 0 {
            rect(Box(x: b.x, y: b.y - antennaLift, w: b.w, h: b.h), antenna ?? p.worker)
        }
    }

    /// A small grey sigh drifting up from the flower being tended.
    static func drawSparkle(ctx: GraphicsContext, x: CGFloat, base: CGFloat, phase: Double) {
        let u: CGFloat = 2
        let rise = CGFloat(Int(phase * 6)) * u
        let shown = min(3, Int(phase * 9))
        var c = ctx
        c.opacity = phase > 0.7 ? max(0, (1 - phase) / 0.3) : 1
        for k in 0..<shown {
            c.fill(Path(CGRect(x: (x - 5).rounded() + CGFloat(k) * 2 * u, y: base - rise, width: u, height: u)), with: .color(Color(hex: 0x8C8C86)))
        }
    }
}

// ── The checkpoint ───────────────────────────────────────────────────────────────────────────
/// What the officer did with the bot at the barrier. It changes only once the server has ADMITTED
/// the decision (DeviceRedeemed): a refused or failed redemption leaves the bot waiting.
enum Verdict: Equatable { case pending, approved, denied }

/// The customs booth, seen from inside it (operator direction 2026-11-02, after Papers, Please): the
/// yard outside with the line of bots waiting, then the window, where the bot at the counter faces
/// you. Bots are short: at rest only the antenna and the top of the head show over the counter, a
/// walking step bobs the eyes into view, and a verdict makes the bot pop up so its face is seen. The
/// antenna ball lights with the outcome, as the workspace's does. Pure in time: the bot's walk-in
/// runs from `arrivedAt`, its reaction from `verdictAt`.
struct CheckpointScene: View {
    let verdict: Verdict
    let verdictAt: Date?
    let arrivedAt: Date
    let queue: Int
    /// False when nothing is pending: the window is empty.
    var occupied = true
    /// Bots approved this session, at work in the garden beyond the fence.
    var recruits = 0
    /// The status bar's height: the scene runs under it, and the yard starts below it.
    var topInset: CGFloat = 0
    /// The request at the window, and the ones in line behind it, in order: each bot is dressed as
    /// its request (Wardrobe), so the next to step up is the next request.
    var visitor: String?
    var line: [String] = []
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.displayScale) private var displayScale

    var body: some View {
        TimelineView(.animation(minimumInterval: 1.0 / 12)) { timeline in
            Canvas { ctx, size in
                Checkpoint.draw(ctx: ctx, size: size, now: timeline.date, verdict: verdict, verdictAt: verdictAt, arrivedAt: arrivedAt,
                                queue: queue, occupied: occupied, recruits: recruits, topInset: topInset, visitor: visitor, line: line, scale: displayScale, instant: reduceMotion)
            }
        }
        .accessibilityElement()
        .accessibilityLabel(accessibilityText)
    }

    private var accessibilityText: String {
        if !occupied { return "The window is empty; nobody is waiting" }
        let line = queue > 0 ? ", \(queue) more in line outside" : ""
        switch verdict {
        case .pending: return "A bot waits at your window with its papers\(line)"
        case .approved: return "Approved: the bot goes through to work\(line)"
        case .denied: return "Denied: the bot is turned back\(line)"
        }
    }
}

enum Checkpoint {
    /// The booth's own palette, the same in light and dark mode: it is a picture, not chrome.
    enum C {
        static let sky = Color(hex: 0x9A9C94), yard = Color(hex: 0x84867F), yardLine = Color(hex: 0x74766F), meadow = Color(hex: 0xA2AA97)
        static let wallTop = Color(hex: 0x5E605A), door = Color(hex: 0x3E403B), doorFrame = Color(hex: 0x2E302C)
        static let near = Color(hex: 0x7C7E78), farWall = Color(hex: 0x666862)
        static let frame = Color(hex: 0x3C3E3A), counterTop = Color(hex: 0x9A9C96), counterFront = Color(hex: 0x6A6C67)
        static let storm = Color(hex: 0x5E615D), cloud = Color(hex: 0x4E514D), clear = Color(hex: 0xC9DCE3), sun = Color(hex: 0xF1E7C2)
        static let rockBack = Color(hex: 0x6E6A63), rock = Color(hex: 0x58544E), rockLight = Color(hex: 0x8A857C)
        static let hut = Color(hex: 0x6F716B), hutRoof = Color(hex: 0x4A4C48), hutWindow = Color(hex: 0xC9CCC4)
        static let bubble = Color(hex: 0xF4F1EC), bubbleInk = Color(hex: 0x262420)
        // The bots in the booth's light: the workspace's light-page worker.
        static let worker = Garden.Palette(.light)
    }

    static let arriveTime: Double = 1.4
    static let walkSpeed: CGFloat = 120

    /// How the bot at the window moves, in the big view's units. The window view and the little
    /// booth in the yard both draw from this, so the two stay in step.
    struct Motion {
        var arrive: CGFloat = 1        // 0 -> 1 while stepping up
        var lift = 0                   // units the head rises over the counter
        var face: Garden.Face = .tired
        var antenna: Color?
        var exit: Double = 0           // seconds spent walking away after the verdict
        var exitRight = true
    }

    static func motion(t: Double, arrived: Double, since: Double?, verdict: Verdict) -> Motion {
        var m = Motion()
        m.arrive = CGFloat(min(max(arrived / arriveTime, 0), 1))
        if arrived < arriveTime {
            m.lift = Int(arrived * 5) % 2 == 0 ? 4 : 1                      // each step bobs the eyes up
        } else if verdict == .pending {
            m.lift = arrived.truncatingRemainder(dividingBy: 5.0) < 0.35 ? 3 : 0   // the odd hopeful peek
        }
        if let s = since, verdict != .pending {
            if s > 0.15 { m.face = verdict == .approved ? .content : .sad; m.antenna = verdict == .approved ? Garden.success : Garden.stop }
            if verdict == .approved && s > 1.4 { m.face = .tired }          // one moment of relief, then back to work
            if s < 1.1 {
                m.lift = s < 0.25 ? 2 : (verdict == .approved ? (Int(s * 6) % 2 == 0 ? 7 : 5) : 5)
            } else {
                m.exit = s - 1.1
                m.exitRight = verdict == .approved
                m.lift = Int(s * 5) % 2 == 0 ? 4 : 1
            }
        }
        return m
    }

    static func draw(ctx: GraphicsContext, size: CGSize, now: Date, verdict: Verdict, verdictAt: Date?, arrivedAt: Date,
                     queue: Int, occupied: Bool, recruits: Int, topInset: CGFloat, visitor: String?, line: [String], scale: CGFloat, instant: Bool) {
        func outfit(_ i: Int) -> Int? { i < line.count ? Wardrobe.costume(for: line[i]) : nil }
        func mood(_ i: Int) -> Garden.Face { i < line.count ? Wardrobe.mood(for: line[i]) : .tired }
        let visitorCostume = visitor.map { Wardrobe.costume(for: $0) }
        let visitorMood = visitor.map { Wardrobe.mood(for: $0) } ?? .tired
        let t = now.timeIntervalSinceReferenceDate
        let arrived = instant ? 100 : t - arrivedAt.timeIntervalSinceReferenceDate
        let since = verdictAt.map { instant ? 100 : t - $0.timeIntervalSinceReferenceDate }
        func px(_ v: CGFloat) -> CGFloat { (v * scale).rounded() / scale }
        let W = size.width
        let m = motion(t: t, arrived: arrived, since: since, verdict: verdict)

        // ── The yard from above the booth: the line, the booth, the fence, the garden ──────────
        // The sky runs to the top of the screen, under the status bar and the island.
        let yardTop: CGFloat = 0
        let yardH = topInset + max((size.height - topInset) * 0.3, 84)
        let ground = yardH - 8
        let fenceX = (W * 0.5).rounded()
        let tiny = px(1)
        let horizon = ground - 34

        // The waiting side: clear sky, a weak sun, and the meadow the line stands in.
        ctx.fill(Path(CGRect(x: 0, y: yardTop, width: fenceX, height: horizon - yardTop)), with: .color(C.clear))
        ctx.fill(Path(ellipseIn: CGRect(x: 16, y: topInset + 6, width: 14, height: 14)), with: .color(C.sun))
        ctx.fill(Path(CGRect(x: 0, y: horizon, width: fenceX, height: yardH - horizon)), with: .color(C.meadow))
        Garden.drawGrass(ctx: ctx, width: fenceX, ground: ground, palette: Garden.Palette(.light))
        let fu = px(2)
        for f in Garden.flowers(width: fenceX - 54, pitch: 24) {
            Garden.drawFlower(ctx: ctx, art: Garden.flowerArt[f.kind], colours: Garden.petals[f.colour], palette: Garden.Palette(.light),
                              centreX: f.x, base: ground, unit: fu)
        }

        // The work side, through the gate: wind, rain, and the quarry, where the approved go.
        let work = CGRect(x: fenceX, y: yardTop, width: W - fenceX, height: yardH - yardTop)
        var weather = ctx
        weather.clip(to: Path(work))
        weather.fill(Path(CGRect(x: fenceX, y: yardTop, width: W - fenceX, height: horizon - yardTop)), with: .color(C.storm))
        let drift = CGFloat((t * 6).truncatingRemainder(dividingBy: 30))
        for k in 0..<Int(work.width / 30) + 2 {
            let cx = fenceX + CGFloat(k) * 30 + drift - 30
            weather.fill(Path(ellipseIn: CGRect(x: cx, y: yardTop + 2 + CGFloat(k % 2) * 4, width: 34, height: 12)), with: .color(C.cloud))
        }
        weather.fill(Path(CGRect(x: fenceX, y: horizon, width: work.width, height: yardH - horizon)), with: .color(C.yard))
        // The booth, in the weather just past the gate, its window toward the line; the officer in it.
        let hut = CGRect(x: fenceX + 6, y: ground - 34, width: 34, height: 34)
        weather.fill(Path(hut), with: .color(C.hut))
        weather.fill(Path(CGRect(x: hut.minX - 3, y: hut.minY - 4, width: hut.width + 6, height: 5)), with: .color(C.hutRoof))
        let hutWin = CGRect(x: hut.minX + 6, y: hut.minY + 7, width: 22, height: 11)
        weather.fill(Path(hutWin), with: .color(C.hutWindow))
        var inHut = weather
        inHut.clip(to: Path(hutWin))
        Garden.drawWorker(ctx: inHut, palette: C.worker, x: hutWin.midX, base: hutWin.maxY + 9, facingLeft: true, lift: 0,
                          blink: t.truncatingRemainder(dividingBy: 3.7) < 0.15, unit: tiny, face: .tired)

        // The work past the booth, all on the same ground, everyone in irons: the regulars' jobs,
        // then this session's approved bots, antennas still lit, put to whatever is next.
        let yardFrom = hut.maxX + 12, yardTo = W - 8
        // The regulars mine and cart; approved bots are put to sweeping and hauling.
        var jobs: [Job] = [.pick, .cart]
        let rota: [Job] = [.sweep, .haul]
        for r in 0..<min(recruits, 2) { jobs.append(rota[r % rota.count]) }
        // The rock face at the far end, and the pile the cart and the hauler feed by the booth.
        weather.fill(Path(CGRect(x: yardTo - 12, y: ground - 18, width: 12, height: 18)), with: .color(C.rock))
        weather.fill(Path(CGRect(x: yardTo - 10, y: ground - 18, width: 8, height: 2)), with: .color(C.rockLight))
        weather.fill(Path(CGRect(x: yardFrom, y: ground - 5, width: 12, height: 5)), with: .color(C.rock))
        weather.fill(Path(CGRect(x: yardFrom + 3, y: ground - 8, width: 6, height: 3)), with: .color(C.rock))
        let minerX = px(yardTo - 34)
        let runFrom = yardFrom + 26, runTo = minerX - 30
        for (k, j) in jobs.enumerated() {
            let lit = k >= 2 ? Garden.success : nil
            let phase = t + Double(k) * 1.9
            switch j {
            case .pick:
                Garden.drawWorker(ctx: weather, palette: C.worker, x: minerX, base: ground, facingLeft: false, lift: 0, blink: false,
                                  unit: tiny, face: .tired, antenna: lit)
                drawPick(ctx: weather, x: minerX, base: ground, facingLeft: false, struck: Int(phase * 2.5) % 2 == 1, t: t)
                drawIrons(ctx: weather, x: minerX, base: ground, facingLeft: false)
            case .sweep:
                let x = px(yardFrom + 22)
                Garden.drawWorker(ctx: weather, palette: C.worker, x: x, base: ground, facingLeft: false, lift: 0, blink: false,
                                  unit: tiny, face: .tired, antenna: lit)
                drawBroom(ctx: weather, x: x, base: ground, phase: phase)
                drawIrons(ctx: weather, x: x, base: ground, facingLeft: false)
            case .cart, .haul:
                // Back and forth between the pile and the miner, slowly.
                let run = Double(max(runTo - runFrom, 1))
                let along = (phase * (j == .cart ? 9 : 12)).truncatingRemainder(dividingBy: 2 * run)
                let outbound = along < run                        // toward the miner, empty
                let x = px(runFrom + CGFloat(outbound ? along : 2 * run - along))
                let step = Int(phase * 5) % 2
                Garden.drawWorker(ctx: weather, palette: C.worker, x: x, base: ground, facingLeft: !outbound, lift: step, blink: false,
                                  unit: tiny, face: .tired, antenna: lit)
                if j == .cart {
                    drawCart(ctx: weather, x: x + (outbound ? 16 : -16), base: ground, loaded: !outbound)
                } else if !outbound {
                    weather.fill(Path(CGRect(x: x - 5, y: ground - 30 - CGFloat(step), width: 10, height: 6)), with: .color(C.rock))
                }
                drawIrons(ctx: weather, x: x, base: ground, facingLeft: !outbound)
            }
        }

        // The fence: a short post, wire strung along it, and a coil of barbed wire on top.
        let postTop = ground - 34
        ctx.fill(Path(CGRect(x: fenceX - 2, y: postTop, width: 4, height: ground - postTop)), with: .color(C.doorFrame))
        for wy in stride(from: postTop + 6, to: ground - 4, by: 8) {
            ctx.fill(Path(CGRect(x: fenceX - 6, y: wy, width: 12, height: 1)), with: .color(C.doorFrame))
        }
        let coil = Color(hex: 0x4A4C48)
        for (k, dx) in [CGFloat(-5), 0, 5].enumerated() {
            let c = CGRect(x: fenceX + dx - 6, y: postTop - 11 - CGFloat(k % 2) * 2, width: 12, height: 11)
            ctx.stroke(Path(ellipseIn: c), with: .color(coil), lineWidth: 1)
        }
        for (bx, by) in [(CGFloat(-10), CGFloat(-6)), (-4, -13), (3, -12), (9, -5), (-1, -1)] {
            var barb = Path()
            barb.move(to: CGPoint(x: fenceX + bx - 1.5, y: postTop + by - 1.5))
            barb.addLine(to: CGPoint(x: fenceX + bx + 1.5, y: postTop + by + 1.5))
            barb.move(to: CGPoint(x: fenceX + bx + 1.5, y: postTop + by - 1.5))
            barb.addLine(to: CGPoint(x: fenceX + bx - 1.5, y: postTop + by + 1.5))
            ctx.stroke(barb, with: .color(coil), lineWidth: 1)
        }

        // The line, and the bot at the gate facing the booth's window: seen from the side here, so
        // its face is the officer's to see (below). The same bot as in the big view, in step.
        let windowX = fenceX - 12
        func spot(_ i: Int) -> CGFloat { windowX - 24 * CGFloat(i + 1) }
        let shuffle = occupied ? m.arrive : 1
        for i in 0..<min(queue, 12) {
            let x = spot(i + 1) + (spot(i) - spot(i + 1)) * shuffle
            guard x > -20 else { break }
            let lift = shuffle < 1 ? Int(arrived * 6 + Double(i)) % 2 : (Int(t * 1.5 + Double(i) * 0.7) % 5 == 0 ? 1 : 0)
            Garden.drawWorker(ctx: ctx, palette: C.worker, x: px(x), base: ground, facingLeft: false, lift: lift,
                              blink: (t + Double(i)).truncatingRemainder(dividingBy: 4.3) < 0.15, unit: tiny, face: mood(i), costume: outfit(i))
        }
        if occupied {
            var x = spot(0) + (windowX - spot(0)) * m.arrive
            var facingLeft = false
            var face: Garden.Face = .back
            if m.exit > 0 {
                // Approved: through the gate, past the booth, to the quarry. Denied: back past the line.
                x = m.exitRight ? windowX + 40 * CGFloat(m.exit) : windowX - 40 * CGFloat(m.exit)
                facingLeft = !m.exitRight
                if !m.exitRight { face = m.face }
            }
            let tinyLift = m.lift >= 5 ? 2 : (m.lift >= 3 ? 1 : 0)
            if x < W + 20 && x > -20 {
                Garden.drawWorker(ctx: ctx, palette: C.worker, x: px(x), base: ground - CGFloat(tinyLift) * tiny, facingLeft: facingLeft,
                                  lift: 0, blink: false, unit: tiny, face: face == .tired ? visitorMood : face, antenna: m.antenna, costume: visitorCostume)
            }
        }
        // Wind and rain over the work side: the booth, the quarry, and whoever has just gone through.
        if !instant { drizzle(ctx: ctx, rect: work, t: t * 1.6, density: 7, length: 8, opacity: 0.5, slant: 1.2) }

        // ── The officer's view out of the booth window: the bot at the counter, the line behind it
        //    in the sunshine, rain on the glass ───────────────────────────────────────────────────
        let view = CGRect(x: 0, y: yardH, width: W, height: size.height - yardH)
        let counterTop = size.height - 44
        let far = view.minY + (counterTop - view.minY) * 0.45
        ctx.fill(Path(view), with: .color(C.meadow))
        ctx.fill(Path(CGRect(x: 0, y: view.minY, width: W, height: far - view.minY)), with: .color(C.clear))
        ctx.fill(Path(ellipseIn: CGRect(x: W * 0.15, y: view.minY + 10, width: 18, height: 18)), with: .color(C.sun))
        ctx.fill(Path(CGRect(x: 0, y: far, width: W, height: 3)), with: .color(Garden.Palette(.light).grass))
        // The line, far off at the horizon, facing the booth.
        let distant = px(1.5)
        let lineX = W * 0.36
        let lineBase = far + 14
        for i in 0..<min(queue, 7) {
            let x = lineX - CGFloat(i + 1) * 46 + 46 * shuffle
            Garden.drawWorker(ctx: ctx, palette: C.worker, x: px(x), base: lineBase, facingLeft: false,
                              lift: Int(t * 1.5 + Double(i) * 0.7) % 5 == 0 ? 1 : 0,
                              blink: (t + Double(i)).truncatingRemainder(dividingBy: 4.3) < 0.15, unit: distant, face: mood(i), costume: outfit(i))
        }
        for f in Garden.flowers(width: W, pitch: 40) {
            Garden.drawFlower(ctx: ctx, art: Garden.flowerArt[f.kind], colours: Garden.petals[f.colour], palette: Garden.Palette(.light),
                              centreX: f.x, base: far + 18, unit: px(3))
        }
        if !occupied {
            ctx.draw(Text("CLOSED FOR NOTHING").font(.system(size: 11, weight: .heavy, design: .monospaced)).foregroundColor(C.bubbleInk.opacity(0.35)),
                     at: CGPoint(x: W / 2, y: (far + counterTop) / 2))
        }
        let u = px(min(7, max(5, (counterTop - view.minY) / 26)))
        let restBase = counterTop + CGFloat(15) * u        // the counter cuts the head just above the eyes
        let centre = W / 2
        if occupied {
            // Stepping up from the front of the line: it grows from the horizon to the window.
            let f = m.arrive
            var x = lineX + (centre - lineX) * f
            var unit = px(distant + (u - distant) * f)
            var base = lineBase + (restBase - CGFloat(m.lift) * u - lineBase) * f
            var face = m.face
            var facingLeft = false
            if m.exit > 0 {
                if m.exitRight {
                    // Approved: past the window, out to the quarry.
                    x = centre + walkSpeed * CGFloat(m.exit)
                } else {
                    // Denied: it turns its back and goes, shrinking toward the line it came from.
                    let g = CGFloat(min(m.exit / 1.6, 1))
                    x = centre + (lineX - 40 - centre) * g
                    unit = px(u + (distant - u) * g)
                    base = (restBase - CGFloat(m.lift) * u) + (lineBase - (restBase - CGFloat(m.lift) * u)) * g
                    face = .back
                    facingLeft = true
                }
            }
            let blink = face == .tired && (t + 0.4).truncatingRemainder(dividingBy: 3.1) < 0.15
            Garden.drawWorker(ctx: ctx, palette: C.worker, x: px(x), base: base, facingLeft: facingLeft,
                              lift: 0, blink: blink, unit: unit, face: face == .tired ? visitorMood : face, antenna: m.antenna,
                              costume: visitorCostume)
        }
        // Rain on the booth's window, between the officer and everything out there.
        if !instant { drizzle(ctx: ctx, rect: CGRect(x: 0, y: view.minY, width: W, height: counterTop - view.minY), t: t * 0.7, density: 13, length: 11, opacity: 0.32, slant: 0.5) }
        // The window frame and the counter.
        ctx.fill(Path(CGRect(x: 0, y: yardH - 3, width: W, height: 6)), with: .color(C.frame))
        ctx.fill(Path(CGRect(x: 0, y: counterTop, width: W, height: size.height - counterTop)), with: .color(C.counterFront))
        ctx.fill(Path(CGRect(x: 0, y: counterTop, width: W, height: 8)), with: .color(C.counterTop))

        // The officer's word as the bot steps up.
        if occupied && arrived > 0.6 && arrived < 3.0 && verdict == .pending {
            let bubble = CGRect(x: W - 84, y: view.minY + 14, width: 64, height: 24)
            ctx.fill(Path(roundedRect: bubble, cornerRadius: 4), with: .color(C.bubble))
            var tail = Path()
            tail.move(to: CGPoint(x: bubble.maxX - 18, y: bubble.maxY))
            tail.addLine(to: CGPoint(x: bubble.maxX - 8, y: bubble.maxY + 8))
            tail.addLine(to: CGPoint(x: bubble.maxX - 28, y: bubble.maxY))
            ctx.fill(tail, with: .color(C.bubble))
            ctx.draw(Text("Next.").font(.system(size: 12, weight: .semibold, design: .monospaced)).foregroundColor(C.bubbleInk),
                     at: CGPoint(x: bubble.midX, y: bubble.midY))
        }
    }

    enum Job { case pick, cart, haul, sweep }

    /// A ball and chain at the ankle.
    static func drawIrons(ctx: GraphicsContext, x: CGFloat, base: CGFloat, facingLeft: Bool) {
        let dir: CGFloat = facingLeft ? 1 : -1                 // the ball drags behind
        let ankle = CGPoint(x: x + dir * 6, y: base - 1)
        let ball = CGPoint(x: x + dir * 15, y: base - 3)
        var chain = Path()
        chain.move(to: ankle)
        chain.addLine(to: ball)
        ctx.stroke(chain, with: .color(Color(hex: 0x3A3B38)), style: StrokeStyle(lineWidth: 1, dash: [1.5, 1]))
        ctx.fill(Path(ellipseIn: CGRect(x: ball.x - 3, y: ball.y - 3, width: 6, height: 6)), with: .color(Color(hex: 0x2A2B28)))
    }

    /// A mine cart: a grey tub on two wheels, heaped with rock when loaded.
    static func drawCart(ctx: GraphicsContext, x: CGFloat, base: CGFloat, loaded: Bool) {
        ctx.fill(Path(CGRect(x: x - 8, y: base - 10, width: 16, height: 7)), with: .color(Color(hex: 0x4A4C48)))
        ctx.fill(Path(CGRect(x: x - 8, y: base - 10, width: 16, height: 1)), with: .color(Color(hex: 0x6A6C67)))
        for wx in [x - 5, x + 5] {
            ctx.fill(Path(ellipseIn: CGRect(x: wx - 2, y: base - 4, width: 4, height: 4)), with: .color(Color(hex: 0x2A2B28)))
        }
        if loaded {
            for (rx, rw) in [(x - 6, CGFloat(5)), (x - 1, 6), (x + 3, 4)] {
                ctx.fill(Path(CGRect(x: rx, y: base - 13, width: rw, height: 3)), with: .color(C.rock))
            }
        }
    }

    /// A broom, swept side to side.
    static func drawBroom(ctx: GraphicsContext, x: CGFloat, base: CGFloat, phase: Double) {
        let lean: CGFloat = Int(phase * 3) % 2 == 0 ? -4 : 4
        let hand = CGPoint(x: x + 9, y: base - 10)
        let foot = CGPoint(x: x + 13 + lean, y: base - 2)
        var stick = Path()
        stick.move(to: hand)
        stick.addLine(to: foot)
        ctx.stroke(stick, with: .color(Color(hex: 0x6B5236)), lineWidth: 1.5)
        ctx.fill(Path(CGRect(x: foot.x - 3, y: base - 3, width: 7, height: 3)), with: .color(Color(hex: 0xA8925E)))
    }

    /// A pickaxe in a miner's hands: a long handle from the hands, and a curved head with a point at
    /// each end set across its tip. Raised over the shoulder, or brought down onto the rock, where a
    /// chip flies off.
    static func drawPick(ctx: GraphicsContext, x: CGFloat, base: CGFloat, facingLeft: Bool, struck: Bool, t: Double) {
        let dir: CGFloat = facingLeft ? -1 : 1
        let hand = CGPoint(x: x + dir * 8, y: base - 10)
        // The handle's direction: up and back over the shoulder, or forward and down onto the rock.
        let angle: CGFloat = struck ? 0.35 : -1.9                 // radians from horizontal, toward `dir`
        let len: CGFloat = 15
        let ux = cos(angle) * dir, uy = sin(angle)
        let tip = CGPoint(x: hand.x + ux * len, y: hand.y + uy * len)
        var handle = Path()
        handle.move(to: hand)
        handle.addLine(to: tip)
        ctx.stroke(handle, with: .color(Color(hex: 0x6B5236)), style: StrokeStyle(lineWidth: 1.6, lineCap: .round))
        // The head: across the handle, bowed back toward the hands, pointed at both ends.
        let px = -uy, py = ux                                     // perpendicular to the handle
        let half: CGFloat = 5
        var head = Path()
        head.move(to: CGPoint(x: tip.x + px * half, y: tip.y + py * half))
        head.addQuadCurve(to: CGPoint(x: tip.x - px * half, y: tip.y - py * half),
                          control: CGPoint(x: tip.x + ux * 1.2, y: tip.y + uy * 1.2))
        ctx.stroke(head, with: .color(Color(hex: 0x8E9089)), style: StrokeStyle(lineWidth: 2, lineCap: .round))
        if struck {
            let f = CGFloat((t * 2.5).truncatingRemainder(dividingBy: 1))
            ctx.fill(Path(CGRect(x: tip.x + dir * (4 + 6 * f), y: base - 4 - 6 * f, width: 2, height: 2)), with: .color(C.rockLight))
        }
    }

    static func drizzle(ctx: GraphicsContext, rect: CGRect, t: Double, density: CGFloat, length: CGFloat = 6, opacity: Double = 0.45, slant: CGFloat = 0.33) {
        var c = ctx
        c.clip(to: Path(rect))
        for k in 0..<Int(rect.width / density) + 1 {
            let seed = Double(k * 7919 % 97) / 97
            let fall = (t * 1.6 + seed).truncatingRemainder(dividingBy: 1)
            let rx = rect.minX + CGFloat(k) * density + CGFloat(seed) * 7 - CGFloat(fall) * 30 * slant
            let ry = rect.minY + CGFloat(fall) * rect.height
            var drop = Path()
            drop.move(to: CGPoint(x: rx, y: ry))
            drop.addLine(to: CGPoint(x: rx - length * slant, y: ry + length))
            c.stroke(drop, with: .color(Color(hex: 0xC8CCC9).opacity(opacity)), lineWidth: 1)
        }
    }
}

// ── The wardrobe: each request's bot ───────────────────────────────────────────────────────
/// Every request is a bot dressed its own way: a costume and a mood, chosen from its escalation id
/// (FNV-1a), so the same request is always the same bot. The idea is the mark studio's
/// (gunbc.design.brand_mark_studio, branch session/swift-badger-498-studio: costume x expression as
/// independent axes, parts attached to the head): Cat, Devil, Trick-or-treater and Santa are its
/// costumes, redrawn at the worker's pixel grain; the rest are work and prison wear in the same spirit.
/// Parts are boxes in the worker's 20-unit frame (body rows 0-15, so a hat sits at negative y).
enum Wardrobe {
    static let body: UInt32 = 0x1000001          // paint the part in the body's own paint
    static let feature: UInt32 = 0x1000002       // or in the face's
    struct Part { let box: Garden.Box; let paint: UInt32; var front = false }
    struct Costume { let name: String; let antennaLift: Int; let parts: [Part] }

    private static func b(_ x: Int, _ y: Int, _ w: Int, _ h: Int, _ paint: UInt32, front: Bool = false) -> Part {
        Part(box: Garden.Box(x: x, y: y, w: w, h: h), paint: paint, front: front)
    }

    static let costumes: [Costume] = [
        Costume(name: "Standard issue", antennaLift: 0, parts: []),
        Costume(name: "Cat ears", antennaLift: 0, parts: [
            b(2, -1, 4, 1, body), b(3, -2, 3, 1, body), b(3, -3, 2, 1, body), b(3, -4, 1, 1, body),
            b(14, -1, 4, 1, body), b(15, -2, 3, 1, body), b(15, -3, 2, 1, body), b(16, -4, 1, 1, body),
            b(4, -2, 1, 1, 0xB89AA0), b(15, -2, 1, 1, 0xB89AA0),
            b(0, 9, 3, 1, feature, front: true), b(17, 9, 3, 1, feature, front: true)]),
        Costume(name: "Devil horns", antennaLift: 0, parts: [
            b(3, -1, 3, 1, 0x9E3B33), b(2, -2, 2, 1, 0x9E3B33), b(2, -3, 1, 1, 0x9E3B33),
            b(14, -1, 3, 1, 0x9E3B33), b(16, -2, 2, 1, 0x9E3B33), b(17, -3, 1, 1, 0x9E3B33)]),
        Costume(name: "Trick-or-treater", antennaLift: 9, parts: [
            b(-1, -1, 22, 1, 0x3C3346), b(5, -2, 10, 1, 0x3C3346), b(6, -3, 8, 1, 0x8A6E3A), b(7, -4, 6, 1, 0x3C3346),
            b(8, -5, 5, 1, 0x3C3346), b(9, -6, 3, 1, 0x3C3346), b(10, -7, 3, 1, 0x3C3346), b(12, -8, 2, 1, 0x3C3346)]),
        Costume(name: "Santa", antennaLift: 6, parts: [
            b(1, -1, 18, 2, 0xE8E4DA), b(3, -2, 14, 1, 0xA8433A), b(5, -3, 11, 1, 0xA8433A), b(8, -4, 9, 1, 0xA8433A),
            b(12, -5, 6, 1, 0xA8433A), b(16, -4, 3, 3, 0xE8E4DA)]),
        Costume(name: "Hard hat", antennaLift: 4, parts: [
            b(1, -1, 18, 1, 0xB8963A), b(4, -2, 12, 1, 0xC9A43A), b(5, -3, 10, 1, 0xC9A43A), b(7, -4, 6, 1, 0xC9A43A),
            b(9, -4, 2, 3, 0xD9BC62)]),
        Costume(name: "Beanie", antennaLift: 3, parts: [
            b(2, -1, 16, 2, 0x55595E), b(4, -2, 12, 1, 0x6A6E73), b(6, -3, 8, 1, 0x6A6E73)]),
        Costume(name: "Flat cap", antennaLift: 2, parts: [
            b(3, -1, 14, 1, 0x6B5A48), b(4, -2, 11, 1, 0x7D6A55), b(14, -1, 6, 1, 0x5A4B3C)]),
        Costume(name: "Mandatory party hat", antennaLift: 5, parts: [
            b(7, -1, 6, 1, 0x8FA0AE), b(8, -2, 4, 1, 0xC9B27A), b(8, -3, 4, 1, 0x8FA0AE), b(9, -4, 2, 1, 0xC9B27A),
            b(9, -5, 2, 1, 0x8FA0AE)]),
        Costume(name: "Chef's hat", antennaLift: 5, parts: [
            b(4, -1, 12, 1, 0xE0DCD2), b(5, -2, 10, 1, 0xEDE9E0), b(3, -3, 14, 2, 0xEDE9E0), b(4, -5, 12, 1, 0xEDE9E0)]),
        Costume(name: "Convict cap", antennaLift: 3, parts: [
            b(3, -1, 14, 1, 0xE0DCD2), b(3, -2, 14, 1, 0x2A2B28), b(3, -3, 14, 1, 0xE0DCD2)]),
        Costume(name: "Headset", antennaLift: 0, parts: [
            b(3, -1, 14, 1, 0x3A3B38), b(1, 0, 2, 3, 0x3A3B38), b(17, 0, 2, 3, 0x3A3B38),
            b(0, 3, 2, 5, 0x4A4C48), b(18, 3, 2, 5, 0x4A4C48), b(1, 8, 1, 2, 0x3A3B38, front: true), b(2, 9, 5, 1, 0x3A3B38, front: true)]),
        Costume(name: "Necktie", antennaLift: 0, parts: [
            b(9, 9, 2, 1, 0x7E3A36, front: true), b(9, 10, 2, 4, 0x9E4A44, front: true), b(8, 13, 4, 1, 0x9E4A44, front: true)]),
        Costume(name: "Bow tie", antennaLift: 0, parts: [
            b(6, 10, 3, 2, 0x8FA0AE, front: true), b(11, 10, 3, 2, 0x8FA0AE, front: true), b(9, 10, 2, 2, 0x6F7E8A, front: true)]),
        Costume(name: "Reading glasses", antennaLift: 0, parts: [
            b(3, 3, 5, 1, 0xA8A49A, front: true), b(3, 7, 5, 1, 0xA8A49A, front: true), b(3, 3, 1, 5, 0xA8A49A, front: true),
            b(7, 3, 1, 5, 0xA8A49A, front: true), b(12, 3, 5, 1, 0xA8A49A, front: true), b(12, 7, 5, 1, 0xA8A49A, front: true),
            b(12, 3, 1, 5, 0xA8A49A, front: true), b(16, 3, 1, 5, 0xA8A49A, front: true), b(8, 4, 4, 1, 0xA8A49A, front: true)]),
        Costume(name: "Sticking plaster", antennaLift: 0, parts: [
            b(13, 1, 4, 1, 0xD9C7A0), b(14, 0, 2, 3, 0xD9C7A0)]),
        Costume(name: "Hi-vis sash", antennaLift: 0, parts: [
            b(3, 8, 3, 1, 0xC9A43A, front: true), b(5, 9, 3, 1, 0xC9A43A, front: true), b(7, 10, 3, 1, 0xC9A43A, front: true),
            b(9, 11, 3, 1, 0xC9A43A, front: true), b(11, 12, 3, 1, 0xC9A43A, front: true), b(13, 13, 3, 1, 0xC9A43A, front: true)]),
    ]

    static let moods: [(Garden.Face, String)] = [(.tired, "tired"), (.lethargic, "lethargic"), (.moody, "moody"), (.angry, "angry"), (.depressed, "depressed")]

    static func hash(_ id: String) -> UInt64 {
        var h: UInt64 = 0xcbf29ce484222325
        for b in id.utf8 { h = (h ^ UInt64(b)) &* 0x100000001b3 }
        return h
    }
    static func costume(for id: String) -> Int { Int(hash(id) % UInt64(costumes.count)) }
    static func mood(for id: String) -> Garden.Face { moods[Int((hash(id) / UInt64(costumes.count)) % UInt64(moods.count))].0 }
    static func describe(_ id: String) -> String {
        let m = moods[Int((hash(id) / UInt64(costumes.count)) % UInt64(moods.count))].1
        return "\(costumes[costume(for: id)].name), \(m)"
    }

    /// Where the request comes from: the escalation id's leading words, before its hashes and run.
    static func origin(of id: String) -> String {
        var words: [Substring] = []
        for w in id.split(separator: "-") {
            if w == "run" || (w.count >= 8 && w.allSatisfy(\.isHexDigit)) { break }
            words.append(w)
        }
        return words.isEmpty ? id : words.joined(separator: "-")
    }
}

/// A request's bot, head and shoulders, in its costume and mood: its photo on the request.
struct BotPortrait: View {
    let id: String
    @Environment(\.displayScale) private var displayScale
    var body: some View {
        Canvas { ctx, size in
            // Fit the figure from its hat's top (antenna included) down to its shoulders.
            let outfit = Wardrobe.costumes[Wardrobe.costume(for: id)]
            let tall = CGFloat(5 + outfit.antennaLift + 14)
            let fit = min(size.width / 24, size.height / tall)
            let u = max((fit * displayScale).rounded(.down), 1) / displayScale
            let top = (size.height - tall * u) / 2
            Garden.drawWorker(ctx: ctx, palette: Checkpoint.C.worker, x: size.width / 2,
                              base: top + CGFloat(5 + outfit.antennaLift + 18) * u, facingLeft: false, lift: 0, blink: false,
                              unit: u, face: Wardrobe.mood(for: id), costume: Wardrobe.costume(for: id))
        }
        .accessibilityLabel(Wardrobe.describe(id))
    }
}

extension Color {
    init(hex: UInt32) {
        self.init(red: Double((hex >> 16) & 0xFF) / 255, green: Double((hex >> 8) & 0xFF) / 255, blue: Double(hex & 0xFF) / 255)
    }
}
