import QtQuick
import omatro
import "Labels.js" as Labels

// Une carte à jouer : cœurs et carreaux dans la couleur d'alerte du thème,
// piques et trèfles dans sa couleur de texte.
Item {
  id: root

  property var card: null
  property bool selected: false
  property bool dimmed: false
  property bool interactive: true
  property string shortcut: ""
  signal clicked()

  width: Style.space(84)
  height: Math.round(width * 1.4)

  readonly property color ink: card && Labels.isRed(card.suit) ? Color.urgent : Color.foreground
  readonly property string suitText: card ? Labels.suit(card.suit) : ""
  readonly property string rankText: card ? Labels.rank(card.rank) : ""

  // Rebond de la carte et effet qui s'affiche au-dessus.
  function flash(text, color) {
    popup.text = text
    popup.color = color
    flashAnim.restart()
  }

  Rectangle {
    id: face
    width: parent.width
    height: parent.height
    y: root.selected ? -Style.space(20) : 0
    Behavior on y { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }
    radius: Style.cornerRadius
    color: Qt.tint(Color.background, Util.alpha(Color.foreground, mouse.containsMouse && root.interactive ? 0.12 : 0.06))
    border.width: root.selected ? Style.space(2) : Style.normalBorderWidth
    border.color: root.selected ? Color.accent : Style.normalBorderColor
    opacity: root.dimmed ? 0.4 : 1
    Behavior on opacity { NumberAnimation { duration: 150 } }

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

    Text {
      anchors.centerIn: parent
      text: root.suitText
      color: root.ink
      font.family: Style.font.family
      font.pixelSize: Math.round(root.width * 0.45)
    }

    Column {
      anchors { right: parent.right; bottom: parent.bottom; rightMargin: Style.space(7); bottomMargin: Style.space(5) }
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
  }

  Text {
    anchors { horizontalCenter: parent.horizontalCenter; top: parent.bottom; topMargin: Style.space(6) }
    visible: root.shortcut !== ""
    text: root.shortcut
    color: Color.muted
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

  MouseArea {
    id: mouse
    anchors.fill: face
    enabled: root.interactive
    hoverEnabled: true
    cursorShape: Qt.PointingHandCursor
    onClicked: root.clicked()
  }
}
