import QtQuick
import omatro
import "Drawn.js" as Drawn

// A joker's drawing: the 48 × 48 pixel art of art/jokers/<id>.txt, painted in
// the theme's colours (shaders/palette.frag), its pixels kept sharp.
// `ready` is false for a joker not drawn yet.
Item {
  id: root

  property string jokerId: ""
  readonly property bool drawn: Drawn.ids[jokerId] === true
  readonly property bool ready: drawn && image.status === Image.Ready

  implicitWidth: Style.space(112)
  implicitHeight: implicitWidth

  Image {
    id: image
    visible: false
    smooth: false
    source: root.drawn ? "../art/jokers/" + root.jokerId + ".pgm" : ""
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
    fragmentShader: "../shaders/palette.frag.qsb"
  }
}
