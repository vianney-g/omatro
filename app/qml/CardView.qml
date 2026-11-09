import QtQuick
import QtQuick.Effects
import omatro
import "Labels.js" as Labels

// A playing card: hearts and diamonds in the theme's urgent color, spades and
// clubs in its text color. Its enhancement shows in the suit symbol in the
// middle (SuitMark, which also tints the card in its hue), its seal as a
// symbol in the corner, its edition as a tag at the top; hovering it explains
// them.
Item {
  id: root

  property var card: null
  // Names of the enhancements, seals and editions (`state.modifiers`).
  property var modifiers: null
  property bool selected: false
  property bool dimmed: false
  // Dealt face down by the boss: the card shows its back.
  property bool faceDown: false
  // The boss keeping the card from scoring ("" when none).
  property string disabledBy: ""
  // The Clippy keeps it selected.
  property bool forced: false
  property bool interactive: true
  property string shortcut: ""
  // False when the view shows the details itself (the deck in fastfetch).
  property bool showTip: true
  // The enhancement's band moves (off in views showing the whole deck).
  property bool lively: true
  readonly property bool hovered: mouse.containsMouse
  signal clicked()

  width: Style.space(84)
  height: Math.round(width * 1.4)

  readonly property bool blob: card !== null && card.enhancement === "binary_blob"
  readonly property color ink: blob ? Colors.muted
    : card && Labels.isRed(card.suit) ? Colors.urgent : Colors.foreground
  readonly property string suitText: card && !blob ? Labels.suit(card.suit) : ""
  readonly property string rankText: card && !blob ? Labels.rank(card.rank) : ""

  function info(kind, id) {
    if (!modifiers || !id) return null
    var list = modifiers[kind]
    for (var i = 0; i < list.length; i++)
      if (list[i].id === id) return list[i]
    return null
  }
  readonly property var enhancement: card ? info("enhancements", card.enhancement) : null
  readonly property var seal: card ? info("seals", card.seal) : null
  readonly property var edition: card ? info("editions", card.edition) : null
  readonly property int bonusChips: card && card.bonus_chips ? card.bonus_chips : 0

  readonly property string details: {
    if (faceDown) return I18n.tr("Face down: dealt by the boss.")
    var lines = []
    if (disabledBy) lines.push(I18n.tr("Disabled by {boss}: scores nothing.", { boss: disabledBy }))
    if (forced) lines.push(I18n.tr("The Clippy keeps it selected."))
    if (enhancement) lines.push(enhancement.name + ": " + enhancement.description)
    if (seal) lines.push(seal.name + " seal: " + seal.description)
    if (edition) lines.push(edition.name + ": " + edition.description)
    if (bonusChips) lines.push("+" + bonusChips + " chips gained")
    return lines.join("\n")
  }

  // Card bounce and the effect shown above it; `effect` (a scoring step's)
  // tells when it comes from the enhancement, which then plays its burst.
  function flash(text, color, effect) {
    popup.text = text
    popup.color = color
    flashAnim.restart()
    if (effect !== undefined && fromEnhancement(effect)) {
      suitMark.activate()
      enhancedAnim.restart()
    }
    // The seal acts: Retry as the card scores again, Gold Star as it pays.
    if (card && ((effect === "again" && card.seal === "retry")
        || (typeof effect === "object" && effect !== null && effect.money === 3 && card.seal === "gold_star")))
      sealMark.activate()
  }
  function fromEnhancement(e) {
    if (!card || !card.enhancement || typeof e !== "object" || e === null) return false
    switch (card.enhancement) {
    case "optimized":
    case "binary_blob": return e.chips !== undefined
    case "multithreaded": return e.mult === 4
    case "flaky_test": return e.mult === 20 || e.money === 20
    case "overclocked": return e.x_mult === 2
    case "daemon": return e.x_mult === 1.5
    default: return false
    }
  }
  // How much the card glows in its enhancement's hue (while it scores).
  property real enhancedGlow: 0

  // What a command, a joker or a boss did to the card (Main.showCardChanges):
  // it breaks apart, turns over to its new face, comes in, or falls away.
  property var nextCard: null
  property color glowColor: Colors.accent
  function vanish() { vanishAnim.restart() }
  // An Overclocked card breaks: it overheats, then bursts into pieces.
  function shatter() { shatterAnim.restart() }
  function drop() { dropAnim.restart() }
  function morph(card) {
    nextCard = card
    morphAnim.restart()
  }
  function appear() { appearAnim.restart() }
  // Played: the card comes from where it was in the hand (dx, dy away).
  function flyFrom(dx, dy, delay) {
    fly.x = dx
    fly.y = dy
    flyAnim.delay = delay
    flyAnim.restart()
  }
  SequentialAnimation {
    id: flyAnim
    property int delay: 0
    PauseAnimation { duration: flyAnim.delay }
    ParallelAnimation {
      NumberAnimation { target: fly; property: "x"; to: 0; duration: 260; easing.type: Easing.OutCubic }
      NumberAnimation { target: fly; property: "y"; to: 0; duration: 260; easing.type: Easing.OutCubic }
    }
  }

  // A halo in the enhancement's hue behind the card while it scores: a plate
  // of the card's shape, blurred (drawn only then).
  Rectangle {
    id: glowPlate
    x: face.x; y: face.y
    width: face.width; height: face.height
    radius: Style.cornerRadius
    color: suitMark.hue
    visible: false
  }
  MultiEffect {
    source: glowPlate
    x: face.x; y: face.y
    width: face.width; height: face.height
    scale: face.scale * (1 + 0.16 * root.enhancedGlow)
    visible: root.enhancedGlow > 0
    opacity: root.enhancedGlow
    blurEnabled: true
    blur: 1
    blurMax: 48
    // Strong enough to show on light themes.
    brightness: 0.1
    saturation: 0.4
    autoPaddingEnabled: true
  }

  // The edition's shadow, under the card (EditionLayer).
  EditionLayer {
    x: face.x; y: face.y
    width: face.width; height: face.height
    scale: face.scale
    behind: true
    visible: !root.faceDown
    kind: root.card && root.card.edition ? root.card.edition : ""
  }

  Rectangle {
    id: face
    width: parent.width
    height: parent.height
    y: root.selected ? -Style.space(20) : 0
    Behavior on y { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }
    radius: Style.cornerRadius
    color: Qt.tint(Colors.background, Util.alpha(root.edition ? Colors.accent
        : root.enhancement ? suitMark.hue : Colors.foreground,
      (mouse.containsMouse && root.interactive ? 0.12 : 0.06) + (root.edition || root.enhancement ? 0.08 : 0)))

    border.width: root.selected || root.edition ? Style.space(2) : Style.normalBorderWidth
    border.color: root.selected ? Colors.accent
      : root.edition ? Util.alpha(Colors.accent, 0.6) : Style.normalBorderColor
    opacity: root.dimmed || (root.disabledBy && !root.faceDown) ? 0.45 : 1
    Behavior on opacity { NumberAnimation { duration: 150 } }
    transform: [
      Scale {
        id: flip
        origin.x: face.width / 2
      },
      Translate { id: fly }
    ]

    // The edition, under the card's content.
    EditionLayer {
      anchors.fill: parent
      visible: !root.faceDown
      lively: root.lively
      kind: root.card && root.card.edition ? root.card.edition : ""
    }

    Column {
      x: Style.space(7)
      y: Style.space(5)
      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: root.rankText
        color: root.ink
        font.family: Style.font.family
        font.pixelSize: Style.font.heading
        font.bold: true
      }
      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: root.suitText
        color: root.ink
        font.family: Style.font.family
        font.pixelSize: Style.font.body
      }
    }

    // Seal, top right (SealMark: its motion, its burst as it acts).
    SealMark {
      id: sealMark
      anchors { right: parent.right; top: parent.top; rightMargin: Style.space(6); topMargin: Style.space(4) }
      visible: kind !== "" && !root.faceDown
      kind: root.card && root.card.seal ? root.card.seal : ""
      lively: root.lively
    }

    // The suit symbol in the middle, drawn after the enhancement.
    SuitMark {
      id: suitMark
      anchors.fill: parent
      kind: root.card && root.card.enhancement ? root.card.enhancement : ""
      suit: root.card && !root.blob ? root.card.suit : ""
      text: root.blob ? "BLOB" : root.suitText
      ink: root.ink
      pixelSize: root.blob ? Style.font.heading : Math.round(root.width * 0.45)
      bold: root.blob
      lively: root.lively && !root.faceDown
    }

    Column {
      anchors { right: parent.right; bottom: parent.bottom; rightMargin: Style.space(7); bottomMargin: Style.space(5) }
      visible: root.bonusChips === 0 && root.edition === null
      rotation: 180
      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: root.rankText
        color: root.ink
        font.family: Style.font.family
        font.pixelSize: Style.font.heading
        font.bold: true
      }
      Text {
        anchors.horizontalCenter: parent.horizontalCenter
        text: root.suitText
        color: root.ink
        font.family: Style.font.family
        font.pixelSize: Style.font.body
      }
    }

    // Edition, then the chips gained, at the bottom.
    Column {
      anchors { left: parent.left; right: parent.right; bottom: parent.bottom; margins: Style.space(3) }
      spacing: Style.space(2)

      Text {
        width: parent.width
        visible: root.edition !== null
        horizontalAlignment: Text.AlignHCenter
        fontSizeMode: Text.HorizontalFit
        minimumPixelSize: Style.fontPx(0.6)
        text: root.edition ? root.edition.name : ""
        color: Colors.accent
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
        font.bold: true
      }

      Text {
        width: parent.width
        visible: root.bonusChips > 0
        horizontalAlignment: Text.AlignHCenter
        text: "+" + root.bonusChips
        color: Colors.accent
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
        font.bold: true
      }
    }

    // Disabled by the boss: crossed out.
    Text {
      anchors.centerIn: parent
      visible: root.disabledBy !== "" && !root.faceDown
      text: "✕"
      color: Colors.urgent
      font.family: Style.font.family
      font.pixelSize: Math.round(root.width * 0.6)
      font.bold: true
    }

    // Face down: the back of the card, over everything.
    CardBack {
      anchors.fill: parent
      visible: root.faceDown
    }

    // Lights up while the card changes.
    Rectangle {
      id: glow
      anchors.fill: parent
      opacity: 0
      radius: Style.cornerRadius
      color: Util.alpha(root.glowColor, 0.25)
      border.width: Style.space(3)
      border.color: root.glowColor
    }
  }

  Text {
    anchors { horizontalCenter: parent.horizontalCenter; top: parent.bottom; topMargin: Style.space(6) }
    visible: root.shortcut !== ""
    text: root.shortcut
    color: Colors.muted
    font.family: Style.font.family
    font.pixelSize: Style.font.caption
  }

  Text {
    id: popup
    anchors.horizontalCenter: face.horizontalCenter
    y: face.y - height - Style.space(6)
    opacity: 0
    font.family: Style.font.family
    font.pixelSize: Style.font.heading
    font.bold: true
  }

  SequentialAnimation {
    id: flashAnim
    ParallelAnimation {
      NumberAnimation { target: face; property: "scale"; from: 1; to: 1.12; duration: 90 }
      NumberAnimation { target: popup; property: "opacity"; to: 1; duration: 90 }
    }
    NumberAnimation { target: face; property: "scale"; to: 1; duration: 180; easing.type: Easing.OutBack }
    PauseAnimation { duration: 220 }
    NumberAnimation { target: popup; property: "opacity"; to: 0; duration: 200 }
  }

  // The enhancement scored: the card glows in its hue (Overclocked shakes).
  SequentialAnimation {
    id: enhancedAnim
    NumberAnimation { target: root; property: "enhancedGlow"; to: 1; duration: 90 }
    SequentialAnimation {
      loops: root.card && root.card.enhancement === "overclocked" ? 3 : 0
      NumberAnimation { target: face; property: "rotation"; to: 4; duration: 40 }
      NumberAnimation { target: face; property: "rotation"; to: -4; duration: 80 }
      NumberAnimation { target: face; property: "rotation"; to: 0; duration: 40 }
    }
    NumberAnimation { target: root; property: "enhancedGlow"; to: 0; duration: 500; easing.type: Easing.InQuad }
  }

  // The pieces of a broken card: snapshots of the face, 3 × 4, flying apart
  // (their own way and spin each), falling and fading as `burst` goes 0 → 1.
  Item {
    id: shards
    x: face.x; y: face.y
    width: face.width; height: face.height
    visible: false
    property real burst: 0
    readonly property int columns: 3
    readonly property int rows: 4
    Repeater {
      model: shards.visible ? shards.columns * shards.rows : 0
      ShaderEffectSource {
        required property int index
        readonly property real w: shards.width / shards.columns
        readonly property real h: shards.height / shards.rows
        readonly property real baseX: (index % shards.columns) * w
        readonly property real baseY: Math.floor(index / shards.columns) * h
        // Away from the middle, with some chance in it.
        readonly property real dx: (baseX + w / 2 - shards.width / 2) * (1.5 + Math.random()) + (Math.random() - 0.5) * w
        readonly property real dy: (baseY + h / 2 - shards.height / 2) * (1.2 + Math.random()) - h * Math.random()
        readonly property real spin: (Math.random() - 0.5) * 540
        sourceItem: face
        sourceRect: Qt.rect(baseX, baseY, w, h)
        live: false
        hideSource: shards.visible
        width: w; height: h
        x: baseX + dx * shards.burst
        // Then they fall.
        y: baseY + dy * shards.burst + shards.height * 0.9 * shards.burst * shards.burst
        rotation: spin * shards.burst
        opacity: 1 - shards.burst * shards.burst
      }
    }
  }

  SequentialAnimation {
    id: shatterAnim
    // It overheats: the halo, a shake.
    ScriptAction { script: root.glowColor = Colors.orange }
    NumberAnimation { target: root; property: "enhancedGlow"; to: 1; duration: 80 }
    SequentialAnimation {
      loops: 3
      NumberAnimation { target: face; property: "rotation"; to: 5; duration: 35 }
      NumberAnimation { target: face; property: "rotation"; to: -5; duration: 70 }
      NumberAnimation { target: face; property: "rotation"; to: 0; duration: 35 }
    }
    // It bursts.
    ScriptAction { script: { shards.burst = 0; shards.visible = true; root.enhancedGlow = 0 } }
    NumberAnimation { target: shards; property: "burst"; to: 1; duration: 650; easing.type: Easing.OutQuad }
    ScriptAction { script: { root.opacity = 0; shards.visible = false } }
  }

  SequentialAnimation {
    id: vanishAnim
    ScriptAction { script: root.glowColor = Colors.urgent }
    ParallelAnimation {
      NumberAnimation { target: glow; property: "opacity"; to: 1; duration: 90 }
      NumberAnimation { target: face; property: "scale"; to: 1.1; duration: 90 }
    }
    ParallelAnimation {
      NumberAnimation { target: root; property: "opacity"; to: 0; duration: 340; easing.type: Easing.InCubic }
      NumberAnimation { target: face; property: "scale"; to: 0.4; duration: 340; easing.type: Easing.InBack }
      NumberAnimation { target: face; property: "rotation"; to: 18; duration: 340; easing.type: Easing.InQuad }
    }
  }

  // Thrown out of the hand (The OOM Killer): it falls away, greyed.
  SequentialAnimation {
    id: dropAnim
    ScriptAction { script: root.glowColor = Colors.muted }
    NumberAnimation { target: glow; property: "opacity"; to: 1; duration: 90 }
    ParallelAnimation {
      NumberAnimation { target: fly; property: "y"; to: root.height * 0.8; duration: 380; easing.type: Easing.InCubic }
      NumberAnimation { target: root; property: "opacity"; to: 0; duration: 380; easing.type: Easing.InQuad }
    }
  }

  SequentialAnimation {
    id: morphAnim
    ScriptAction { script: root.glowColor = Colors.accent }
    ParallelAnimation {
      NumberAnimation { target: flip; property: "xScale"; to: 0; duration: 150; easing.type: Easing.InQuad }
      NumberAnimation { target: glow; property: "opacity"; to: 1; duration: 150 }
    }
    ScriptAction { script: if (root.nextCard) root.card = root.nextCard }
    NumberAnimation { target: flip; property: "xScale"; to: 1; duration: 220; easing.type: Easing.OutBack }
    NumberAnimation { target: glow; property: "opacity"; to: 0; duration: 260 }
  }

  SequentialAnimation {
    id: appearAnim
    ScriptAction { script: root.glowColor = Colors.accent }
    ParallelAnimation {
      NumberAnimation { target: root; property: "opacity"; from: 0; to: 1; duration: 200 }
      NumberAnimation { target: face; property: "scale"; from: 0.4; to: 1; duration: 340; easing.type: Easing.OutBack }
      NumberAnimation { target: glow; property: "opacity"; from: 1; to: 1; duration: 200 }
    }
    NumberAnimation { target: glow; property: "opacity"; to: 0; duration: 400 }
  }

  MouseArea {
    id: mouse
    anchors.fill: face
    hoverEnabled: true
    cursorShape: root.interactive ? Qt.PointingHandCursor : Qt.ArrowCursor
    onClicked: if (root.interactive) root.clicked()
  }

  // What the card carries, while hovered.
  Rectangle {
    z: 10
    visible: root.showTip && mouse.containsMouse && root.details !== ""
    anchors { horizontalCenter: face.horizontalCenter; bottom: face.top; bottomMargin: Style.space(8) }
    width: Math.min(Style.space(260), tip.implicitWidth + 2 * Style.space(8))
    height: tip.implicitHeight + 2 * Style.space(6)
    radius: Style.cornerRadius
    color: Colors.background
    border.width: Style.normalBorderWidth
    border.color: Colors.accent
    Text {
      id: tip
      anchors { fill: parent; margins: Style.space(6) }
      wrapMode: Text.WordWrap
      text: root.details
      color: Colors.foreground
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
    }
  }
}
