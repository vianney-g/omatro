import QtQuick
import omatro
import "Labels.js" as Labels

// A playing card: hearts and diamonds in the theme's urgent color, spades and
// clubs in its text color. Its enhancement shows as a band at the bottom, its
// seal as a symbol in the corner, its edition as a tag at the top; hovering
// it explains them.
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

  readonly property string sealSymbol: !card || !card.seal ? ""
    : card.seal === "gold_star" ? "★" : card.seal === "retry" ? "↻"
    : card.seal === "theme_sync" ? "◐" : "❯"
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

  // Card bounce and the effect shown above it.
  function flash(text, color) {
    popup.text = text
    popup.color = color
    flashAnim.restart()
  }

  // What a command, a joker or a boss did to the card (Main.showCardChanges):
  // it breaks apart, turns over to its new face, comes in, or falls away.
  property var nextCard: null
  property color glowColor: Colors.accent
  function vanish() { vanishAnim.restart() }
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

  Rectangle {
    id: face
    width: parent.width
    height: parent.height
    y: root.selected ? -Style.space(20) : 0
    Behavior on y { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }
    radius: Style.cornerRadius
    color: Qt.tint(Colors.background, Util.alpha(root.edition ? Colors.accent : Colors.foreground,
      (mouse.containsMouse && root.interactive ? 0.12 : 0.06) + (root.edition ? 0.08 : 0)))
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

    // Seal, top right.
    Text {
      anchors { right: parent.right; top: parent.top; rightMargin: Style.space(6); topMargin: Style.space(4) }
      visible: root.sealSymbol !== ""
      text: root.sealSymbol
      color: root.card && root.card.seal === "retry" ? Colors.urgent : Colors.accent
      font.family: Style.font.family
      font.pixelSize: Style.font.heading
      font.bold: true
    }

    Text {
      anchors.centerIn: parent
      text: root.blob ? "BLOB" : root.suitText
      color: root.ink
      font.family: Style.font.family
      font.pixelSize: root.blob ? Style.font.heading : Math.round(root.width * 0.45)
      font.bold: root.blob
    }

    Column {
      anchors { right: parent.right; bottom: parent.bottom; rightMargin: Style.space(7); bottomMargin: Style.space(5) }
      visible: band.text === "" && root.edition === null
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

    // Edition, then the enhancement (and chips gained) as a band, at the bottom.
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

      Rectangle {
        width: parent.width
        visible: band.text !== ""
        height: band.implicitHeight + Style.space(4)
        radius: Style.cornerRadius
        color: Util.alpha(Colors.accent, 0.22)
        Text {
          id: band
          anchors { fill: parent; leftMargin: Style.space(2); rightMargin: Style.space(2) }
          horizontalAlignment: Text.AlignHCenter
          verticalAlignment: Text.AlignVCenter
          fontSizeMode: Text.HorizontalFit
          minimumPixelSize: Style.fontPx(0.6)
          text: (root.enhancement ? root.enhancement.name : "")
            + (root.bonusChips ? (root.enhancement ? " " : "") + "+" + root.bonusChips : "")
          color: Colors.foreground
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
          font.bold: true
        }
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
