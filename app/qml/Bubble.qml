import QtQuick
import QtQuick.Shapes
import QtQuick.Effects
import omatro

// A bubble telling about something hovered, styled as the Omarchy shell's
// notifications (the theme's background, an accent border, its rounding),
// with a shadow and an arrow pointing at what it tells about, so that it
// never passes for one more card: beside `place` (on its right, or its left
// when there is no room), or under the y `below` when it is set (≥ 0),
// pointing up at `place`. Always in the window: fill the window with it.
// It takes no pointer.
Item {
  id: root

  // What it points at, in its own coordinates.
  property rect place: Qt.rect(0, 0, 0, 0)
  property real below: -1
  property bool shown: false
  property real panelWidth: Style.space(250)
  default property alias content: column.data

  readonly property real arrowSize: Style.space(8)
  readonly property real border: Style.space(2)
  readonly property real gap: Style.space(4) + arrowSize
  readonly property real padding: Style.space(10)
  readonly property bool downward: below >= 0
  readonly property bool onRight: place.x + place.width + gap + panel.width <= width

  enabled: false
  visible: shown

  RectangularShadow {
    anchors.fill: panel
    radius: panel.radius
    color: Qt.rgba(0, 0, 0, 0.5)
    blur: Style.space(18)
    offset.x: Style.space(3)
    offset.y: Style.space(6)
  }

  Rectangle {
    id: panel
    width: root.panelWidth
    height: column.implicitHeight + 2 * root.padding
    x: root.downward
      ? Math.max(0, Math.min(root.place.x + root.place.width / 2 - width / 2, root.width - width))
      : root.onRight ? root.place.x + root.place.width + root.gap : Math.max(0, root.place.x - root.gap - width)
    y: root.downward
      ? Math.min(root.below + root.gap, root.height - height)
      : Math.max(0, Math.min(root.place.y + root.place.height / 2 - height / 2, root.height - height))
    radius: Style.cornerRadius
    color: Colors.background
    border.width: root.border
    border.color: Colors.accent

    Column {
      id: column
      anchors { left: parent.left; right: parent.right; top: parent.top; margins: root.padding }
      spacing: Style.space(6)
    }
  }

  // The arrow, on the panel's edge facing what it points at: drawn pointing
  // left from its middle (the panel's edge), then turned.
  Item {
    id: arrow
    readonly property real a: root.arrowSize
    readonly property real inset: Math.max(Style.cornerRadius, 0) + a
    width: 2 * a
    height: 2 * a
    x: (root.downward
        ? Math.max(panel.x + inset, Math.min(root.place.x + root.place.width / 2, panel.x + panel.width - inset))
        : root.onRight ? panel.x : panel.x + panel.width) - a
    y: (root.downward
        ? panel.y
        : Math.max(panel.y + inset, Math.min(root.place.y + root.place.height / 2, panel.y + panel.height - inset))) - a
    rotation: root.downward ? 90 : root.onRight ? 0 : 180
    Shape {
      anchors.fill: parent
      preferredRendererType: Shape.CurveRenderer
      // Covers the panel's border where the arrow leaves it.
      ShapePath {
        strokeWidth: -1
        fillColor: Colors.background
        startX: arrow.a + root.border; startY: 0
        PathLine { x: 0; y: arrow.a }
        PathLine { x: arrow.a + root.border; y: 2 * arrow.a }
      }
      ShapePath {
        strokeWidth: root.border
        strokeColor: Colors.accent
        fillColor: "transparent"
        joinStyle: ShapePath.MiterJoin
        capStyle: ShapePath.FlatCap
        startX: arrow.a + root.border / 2; startY: 0
        PathLine { x: root.border / 2; y: arrow.a }
        PathLine { x: arrow.a + root.border / 2; y: 2 * arrow.a }
      }
    }
  }
}
