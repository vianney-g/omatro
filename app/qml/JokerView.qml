import QtQuick
import QtQuick.Shapes
import omatro
import "Labels.js" as Labels

// A joker: rarity, name, effect, what it adds now, its quip and its edition.
// Hovered, it leans towards the pointer in perspective, a light following
// the pointer on it; at rest, it breathes (each joker at its own pace).
Item {
  id: root

  property var joker: null
  // The Kernel Panic: jokers face down.
  property bool hidden: false
  // Hovered, its terms are explained beside it (Glossary).
  // Its stickers are explained too. Narrowed, its full-size view explains
  // them instead (CardPeek).
  property bool hints: true
  HoverHandler {
    id: hover
    onHoveredChanged: if (root.hints) hovered && !root.hidden ? Hints.show(root, root.joker ? root.joker.description
      + root.stickers.map(function(s) { return " " + s.name }).join("") : "", root.joker ? root.joker.name : "") : Hints.hide(root)
  }

  // Welded, Worn, Leased (the higher difficulty levels).
  readonly property var stickers: joker && joker.stickers ? joker.stickers : []
  // A Worn joker past its blinds does nothing.
  readonly property bool wornOut: stickers.some(function(s) { return s.id === "worn" && s.blinds_left === 0 })

  // The edition moves (off where many jokers show at once).
  property bool lively: true

  // The Windows Update: this joker does nothing this hand.
  property bool disabled: false
  // What a growing joker gives right now: {kind, text}.
  property var bonus: joker ? joker.bonus : null

  width: Style.space(120)
  height: Style.space(230)

  readonly property var edition: joker ? joker.edition : null
  readonly property color rarityColor: !joker ? Colors.muted
    : joker.rarity === "extra" ? Colors.accent
    : joker.rarity === "core" ? Colors.muted : Colors.urgent

  // Leaning towards the pointer: from -1 to 1 across the joker, 0 at rest.
  readonly property real leanX: hover.hovered ? Math.max(-1, Math.min(1, (hover.point.position.x - width / 2) / (width / 2))) : 0
  readonly property real leanY: hover.hovered ? Math.max(-1, Math.min(1, (hover.point.position.y - height / 2) / (height / 2))) : 0
  transform: [
    Rotation {
      origin.x: root.width / 2; origin.y: root.height / 2
      axis { x: 0; y: 1; z: 0 }
      angle: 14 * root.leanX
      Behavior on angle { NumberAnimation { duration: 120; easing.type: Easing.OutQuad } }
    },
    Rotation {
      origin.x: root.width / 2; origin.y: root.height / 2
      axis { x: 1; y: 0; z: 0 }
      angle: -10 * root.leanY
      Behavior on angle { NumberAnimation { duration: 120; easing.type: Easing.OutQuad } }
    },
    // Breathing: a slow swell and float.
    Scale {
      origin.x: root.width / 2; origin.y: root.height / 2
      xScale: 1 + 0.012 * root.breath; yScale: xScale
    },
    Translate { y: -Style.space(2) * root.breath }
  ]
  property real breath: 0
  SequentialAnimation on breath {
    running: root.lively && root.visible && !root.hidden
    loops: Animation.Infinite
    // Each joker starts at its own moment, so that they don't breathe together.
    PauseAnimation { duration: Math.random() * 2500 }
    SequentialAnimation {
      loops: Animation.Infinite
      NumberAnimation { to: 1; duration: 1800; easing.type: Easing.InOutSine }
      NumberAnimation { to: 0; duration: 1800; easing.type: Easing.InOutSine }
    }
  }

  function flash(text, color) {
    popup.text = text
    popup.color = color
    flashAnim.restart()
  }

  // The edition's shadow, under the joker (EditionLayer).
  EditionLayer {
    anchors.fill: parent
    scale: face.scale
    rotation: face.rotation
    behind: true
    visible: !root.hidden
    kind: root.edition ? root.edition.id : ""
  }

  Rectangle {
    id: face
    anchors.fill: parent
    radius: Style.cornerRadius
    color: Qt.tint(Colors.background, Util.alpha(Colors.accent, root.edition ? 0.20 : 0.10))
    border.width: root.edition ? Style.space(2) : Style.normalBorderWidth
    border.color: root.edition ? Colors.accent : Util.alpha(Colors.accent, Style.normalBorderAlpha)
    opacity: (root.disabled || root.wornOut) && !root.hidden ? 0.4 : 1

    // The edition, under the joker's text.
    EditionLayer {
      anchors.fill: parent
      visible: !root.hidden
      lively: root.lively
      kind: root.edition ? root.edition.id : ""
    }

    Column {
      anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.space(8) }
      spacing: Style.space(4)

      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        text: root.joker ? I18n.tr(root.joker.rarity_name) : ""
        color: root.rarityColor
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
      }
      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: root.joker ? root.joker.name : ""
        color: Colors.accent
        font.family: Style.font.family
        font.pixelSize: Style.font.subtitle
        font.bold: true
      }
      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        textFormat: Text.StyledText
        text: root.joker ? Labels.highlight(root.joker.description, Hints.terms, root.joker.name) : ""
        color: Colors.foreground
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
      }
      // Fork, Upstream: what it copies.
      Text {
        width: parent.width
        visible: text !== ""
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: root.joker && root.joker.current ? root.joker.current : ""
        color: Colors.accent
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
        font.bold: true
      }
      // Badge: the bonus a growing joker gives right now; pulses when it changes.
      Rectangle {
        id: badge
        anchors.horizontalCenter: parent.horizontalCenter
        visible: root.bonus !== null && root.bonus !== undefined
        readonly property color tint: !visible ? Colors.accent
          : root.bonus.kind === "mult" || root.bonus.kind === "x_mult" ? Colors.urgent : Colors.accent
        width: badgeText.implicitWidth + 2 * Style.space(8)
        height: badgeText.implicitHeight + 2 * Style.space(3)
        radius: height / 2
        color: Util.alpha(tint, 0.18)
        border.width: Style.normalBorderWidth
        border.color: tint
        Text {
          id: badgeText
          anchors.centerIn: parent
          text: badge.visible ? root.bonus.text : ""
          color: badge.tint
          font.family: Style.font.family
          font.pixelSize: Style.font.body
          font.bold: true
          onTextChanged: if (badge.visible) pulse.restart()
        }
        SequentialAnimation {
          id: pulse
          NumberAnimation { target: badge; property: "scale"; to: 1.25; duration: 120 }
          NumberAnimation { target: badge; property: "scale"; to: 1; duration: 220; easing.type: Easing.OutBack }
        }
      }
      Text {
        width: parent.width
        horizontalAlignment: Text.AlignHCenter
        wrapMode: Text.WordWrap
        text: root.joker ? root.joker.quip : ""
        color: Colors.muted
        font.family: Style.font.family
        font.pixelSize: Style.font.caption
        font.italic: true
      }
    }

    // Stickers, above the edition.
    Flow {
      anchors { left: parent.left; right: parent.right; bottom: editionText.visible ? editionText.top : parent.bottom; margins: Style.space(6) }
      spacing: Style.space(3)
      Repeater {
        model: root.stickers
        Rectangle {
          readonly property color tint: modelData.id === "welded" ? Colors.accent : Colors.urgent
          width: stickerText.implicitWidth + 2 * Style.space(5)
          height: stickerText.implicitHeight + 2 * Style.space(2)
          radius: Style.cornerRadius
          color: Util.alpha(tint, 0.18)
          border.width: Style.normalBorderWidth
          border.color: tint
          Text {
            id: stickerText
            anchors.centerIn: parent
            text: modelData.name + (modelData.blinds_left !== null ? " · " + modelData.blinds_left : "")
            color: parent.tint
            font.family: Style.font.family
            font.pixelSize: Style.font.caption
            font.bold: true
          }
        }
      }
    }

    // Edition, at the bottom.
    Text {
      id: editionText
      anchors { left: parent.left; right: parent.right; bottom: parent.bottom; margins: Style.space(6) }
      visible: root.edition !== null
      horizontalAlignment: Text.AlignHCenter
      wrapMode: Text.WordWrap
      text: root.edition ? root.edition.name + " · " + root.edition.description : ""
      color: Colors.accent
      font.family: Style.font.family
      font.pixelSize: Style.font.caption
      font.bold: true
    }

    // The light on it, following the pointer.
    Shape {
      anchors.fill: parent
      visible: hover.hovered && !root.hidden
      opacity: 0.5
      ShapePath {
        strokeWidth: -1
        fillGradient: RadialGradient {
          centerX: hover.point.position.x; centerY: hover.point.position.y
          focalX: centerX; focalY: centerY
          centerRadius: root.width * 0.9
          focalRadius: 0
          GradientStop { position: 0; color: Qt.rgba(1, 1, 1, 0.3) }
          GradientStop { position: 1; color: "transparent" }
        }
        PathRectangle { x: 0; y: 0; width: root.width; height: root.height; radius: Style.cornerRadius }
      }
    }

    // The Kernel Panic: face down.
    Rectangle {
      anchors.fill: parent
      visible: root.hidden
      radius: Style.cornerRadius
      color: Qt.tint(Colors.background, Util.alpha(Colors.urgent, 0.25))
      border.width: Style.space(2)
      border.color: Util.alpha(Colors.urgent, 0.6)
      Text {
        anchors.centerIn: parent
        text: "?"
        color: Colors.urgent
        font.family: Style.font.family
        font.pixelSize: Style.font.displayLarge
        font.bold: true
      }
    }
  }

  // The Windows Update: disabled this hand.
  Text {
    anchors { horizontalCenter: parent.horizontalCenter; bottom: parent.bottom; bottomMargin: Style.space(40) }
    visible: (root.disabled || root.wornOut) && !root.hidden
    text: I18n.tr(root.wornOut ? "worn out" : "disabled")
    color: Colors.urgent
    font.family: Style.font.family
    font.pixelSize: Style.font.subtitle
    font.bold: true
  }

  Text {
    id: popup
    anchors { horizontalCenter: parent.horizontalCenter; top: parent.bottom; topMargin: Style.space(6) }
    opacity: 0
    font.family: Style.font.family
    font.pixelSize: Style.font.heading
    font.bold: true
  }

  SequentialAnimation {
    id: flashAnim
    ParallelAnimation {
      NumberAnimation { target: face; property: "scale"; from: 1; to: 1.1; duration: 90 }
      NumberAnimation { target: face; property: "rotation"; from: 0; to: -4; duration: 90 }
      NumberAnimation { target: popup; property: "opacity"; to: 1; duration: 90 }
    }
    ParallelAnimation {
      NumberAnimation { target: face; property: "scale"; to: 1; duration: 180; easing.type: Easing.OutBack }
      NumberAnimation { target: face; property: "rotation"; to: 0; duration: 180; easing.type: Easing.OutBack }
    }
    PauseAnimation { duration: 220 }
    NumberAnimation { target: popup; property: "opacity"; to: 0; duration: 200 }
  }
}
