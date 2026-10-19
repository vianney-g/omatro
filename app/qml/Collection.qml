import QtQuick
import omatro
import "Labels.js" as Labels

// The collection over everything: a tab per family of items, found ones with
// their text, the others as "?", hidden ones with the achievement that
// unlocks them; and the achievements. C or Escape closes, arrows switch tabs.
Rectangle {
  id: root

  property var collection: null
  property int tab: 0
  signal close()

  color: Util.alpha(Colors.background, 0.96)

  readonly property var families: collection ? collection.families : []
  // The last tab is the achievements.
  readonly property int tabs: families.length + 1
  readonly property bool onAchievements: tab === families.length
  readonly property var items: onAchievements ? (collection ? collection.achievements : [])
    : (families.length ? families[tab].items : [])

  function move(step) { tab = (tab + step + tabs) % tabs; grid.contentY = 0 }
  function scroll(step) {
    grid.contentY = Math.max(0, Math.min(grid.contentHeight - grid.height,
                                         grid.contentY + step * Style.space(120)))
  }

  MouseArea { anchors.fill: parent; onClicked: root.close() }

  component Header: Text {
    color: Colors.muted
    font.family: Style.font.family
    font.pixelSize: Style.font.caption
  }

  Rectangle {
    anchors { fill: parent; margins: Style.spacing.huge }
    radius: Style.cornerRadius
    color: Colors.background
    border.width: Style.normalBorderWidth
    border.color: Colors.accent

    // Clicks inside the panel do not close it.
    MouseArea { anchors.fill: parent }

    Column {
      id: top
      anchors { left: parent.left; right: parent.right; top: parent.top; margins: Style.spacing.huge }
      spacing: Style.spacing.xl

      Row {
        spacing: Style.spacing.xxl
        Text {
          id: title
          text: I18n.tr("collection")
          color: Colors.accent
          font.family: Style.font.family
          font.pixelSize: Style.font.displayLarge
          font.bold: true
        }
        Header {
          anchors.baseline: title.baseline
          text: (root.collection ? I18n.tr("{found} / {total} found", {
              found: Labels.number(root.collection.found), total: Labels.number(root.collection.total) }) + " · " : "")
            + I18n.tr("←/→: tab · ↑/↓: scroll · C or Esc: close")
          font.italic: true
        }
      }

      // The tabs, with what each one found.
      Flow {
        width: parent.width
        spacing: Style.spacing.md
        Repeater {
          model: root.collection ? root.tabs : 0
          Rectangle {
            readonly property bool chosen: index === root.tab
            readonly property bool achievements: index === root.families.length
            readonly property var family: achievements ? null : root.families[index]
            readonly property int done: achievements
              ? root.collection.achievements.filter(function(a) { return a.done }).length : family.found
            readonly property int total: achievements ? root.collection.achievements.length : family.total
            width: tabLabel.implicitWidth + 2 * Style.spacing.lg
            height: tabLabel.implicitHeight + 2 * Style.spacing.sm
            radius: Style.cornerRadius
            color: chosen ? Qt.tint(Colors.background, Util.alpha(Colors.accent, 0.15)) : Style.normalFill
            border.width: chosen ? Style.space(2) : Style.normalBorderWidth
            border.color: chosen ? Colors.accent : Style.normalBorderColor
            Text {
              id: tabLabel
              anchors.centerIn: parent
              text: (achievements ? I18n.tr("Achievements") : family.name) + "  " + done + "/" + total
              color: chosen ? Colors.accent : Colors.foreground
              font.family: Style.font.family
              font.pixelSize: Style.font.caption
            }
            MouseArea {
              anchors.fill: parent
              cursorShape: Qt.PointingHandCursor
              onClicked: { root.tab = index; grid.contentY = 0 }
            }
          }
        }
      }
    }

    GridView {
      id: grid
      anchors {
        left: parent.left; right: parent.right; top: top.bottom; bottom: parent.bottom
        margins: Style.spacing.huge
      }
      clip: true
      model: root.items
      readonly property int columns: Math.max(1, Math.floor(width / Style.space(210)))
      cellWidth: Math.floor(width / columns)
      cellHeight: Style.space(126)
      boundsBehavior: Flickable.StopAtBounds

      delegate: Item {
        width: grid.cellWidth
        height: grid.cellHeight
        Loader {
          anchors { fill: parent; margins: Style.spacing.sm }
          sourceComponent: root.onAchievements ? achievementTile : itemTile
          property var entry: modelData
        }
      }
    }
  }

  // An item: its text once found; "?" (or "?????" for secrets) before; the
  // achievement it waits for when hidden.
  Component {
    id: itemTile
    Rectangle {
      readonly property var e: parent.entry
      readonly property bool found: e.state === "found"
      readonly property color tint: !e.rarity ? Colors.foreground
        : e.rarity === "extra" ? Colors.accent
        : e.rarity === "core" ? Colors.foreground : Colors.urgent
      radius: Style.cornerRadius
      color: found ? Style.normalFill : "transparent"
      border.width: Style.normalBorderWidth
      border.color: e.state === "locked" ? Colors.urgent : Style.normalBorderColor
      opacity: found ? 1 : 0.7

      Column {
        visible: parent.found
        anchors { fill: parent; margins: Style.spacing.lg }
        spacing: Style.spacing.xs
        Text {
          width: parent.width
          elide: Text.ElideRight
          text: e.name || ""
          color: tint
          font.family: Style.font.family
          font.pixelSize: Style.font.body
          font.bold: true
        }
        Text {
          width: parent.width
          wrapMode: Text.WordWrap
          maximumLineCount: 3
          elide: Text.ElideRight
          textFormat: Text.StyledText
          text: e.description ? Labels.highlight(e.description, Hints.terms, e.name) : ""
          color: Colors.foreground
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
        }
        Text {
          width: parent.width
          wrapMode: Text.WordWrap
          maximumLineCount: 2
          elide: Text.ElideRight
          text: e.quip || ""
          color: Colors.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
          font.italic: true
        }
      }

      Column {
        visible: !parent.found
        anchors.centerIn: parent
        width: parent.width - 2 * Style.spacing.lg
        spacing: Style.spacing.sm
        Text {
          anchors.horizontalCenter: parent.horizontalCenter
          text: e.secret ? "?????" : "?"
          color: e.state === "locked" ? Colors.urgent : Colors.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.display
          font.bold: true
        }
        Text {
          width: parent.width
          horizontalAlignment: Text.AlignHCenter
          wrapMode: Text.WordWrap
          text: e.state === "locked"
            ? I18n.tr("Hidden: {achievement}", { achievement: e.achievement }) + "\n" + I18n.tr(e.condition)
            : I18n.tr("Not found yet")
          color: Colors.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
        }
      }

      MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        onContainsMouseChanged: parent.found && e.description
          ? (containsMouse ? Hints.show(parent, e.description, e.name) : Hints.hide(parent))
          : Hints.hide(parent)
      }
    }
  }

  // An achievement: what it takes, where it stands, what it unlocks.
  Component {
    id: achievementTile
    Rectangle {
      readonly property var e: parent.entry
      radius: Style.cornerRadius
      color: e.done ? Qt.tint(Colors.background, Util.alpha(Colors.accent, 0.12)) : Style.normalFill
      border.width: e.done ? Style.space(2) : Style.normalBorderWidth
      border.color: e.done ? Colors.accent : Style.normalBorderColor

      Column {
        anchors { fill: parent; margins: Style.spacing.lg }
        spacing: Style.spacing.xs
        Text {
          width: parent.width
          elide: Text.ElideRight
          text: (e.done ? "✓ " : "") + e.name
          color: e.done ? Colors.accent : Colors.foreground
          font.family: Style.font.family
          font.pixelSize: Style.font.body
          font.bold: true
        }
        Text {
          width: parent.width
          wrapMode: Text.WordWrap
          maximumLineCount: 2
          elide: Text.ElideRight
          text: e.description
          color: Colors.foreground
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
        }
        Text {
          width: parent.width
          elide: Text.ElideRight
          visible: text !== ""
          text: e.goal !== null && !e.done ? Labels.number(e.progress) + " / " + Labels.number(e.goal) : ""
          color: Colors.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
        }
        Text {
          width: parent.width
          elide: Text.ElideRight
          visible: text !== ""
          text: e.unlocks ? I18n.tr("Unlocked: {name}", { name: e.unlocks })
            : e.unlocks_family ? I18n.tr("Unlocks a hidden {family}", { family: I18n.tr(e.unlocks_family) }) : ""
          color: Colors.urgent
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
        }
        Text {
          width: parent.width
          wrapMode: Text.WordWrap
          maximumLineCount: 2
          elide: Text.ElideRight
          text: e.quip
          color: Colors.muted
          font.family: Style.font.family
          font.pixelSize: Style.font.caption
          font.italic: true
        }
      }
    }
  }
}
