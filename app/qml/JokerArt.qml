import QtQuick
import omatro
import "Drawn.js" as Drawn

// A joker's drawing: the pixel art of art/jokers/<id>.txt, painted in
// the theme's colours (shaders/palette.frag), its pixels kept sharp; with
// `family: "oligarchy"`, an oligarchy card's (art/oligarchy, 64 × 40: give it
// that shape). `ready` is false for one not drawn yet.
Item {
  id: root

  property string jokerId: ""
  property string family: "jokers"
  readonly property bool drawn: (family === "oligarchy" ? Drawn.oligarchy : Drawn.ids)[jokerId] === true
  readonly property bool ready: drawn && image.status === Image.Ready

  implicitWidth: Style.space(112)
  implicitHeight: implicitWidth

  Image {
    id: image
    visible: false
    smooth: false
    source: root.drawn ? "../art/" + root.family + "/" + root.jokerId + ".pgm" : ""
  }
  ShaderEffect {
    anchors.fill: parent
    visible: root.ready
    property var source: image
    property color foreground: Colors.foreground
    property color accent: Colors.accent
    property color muted: Colors.muted
    property color background: Colors.background
    property color warm: Colors.warm
    property color urgent: Colors.urgent
    property color green: Colors.green
    property color cyan: Colors.cyan
    property color magenta: Colors.magenta
    property color orange: Colors.orange
    fragmentShader: "../shaders/palette.frag.qsb"
  }
}
