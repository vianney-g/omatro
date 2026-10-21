import QtQuick
import omatro
import "Labels.js" as Labels

// The collection over everything: a tab per family of items, found ones with
// their text, the others as "?", hidden ones with the achievement that
// unlocks them; the achievements; the stats and the history of the games.
// C or Escape closes, arrows switch tabs.
Rectangle {
  id: root

  property var collection: null
  property int tab: 0
  signal close()

  color: Util.alpha(Colors.background, 0.96)

  readonly property var families: collection ? collection.families : []
  readonly property var stats: collection ? collection.stats : null
  readonly property var history: collection ? collection.history : []
  // After the families: the achievements, the stats, the history.
  readonly property int achievementsTab: families.length
  readonly property int statsTab: families.length + 1
  readonly property int historyTab: families.length + 2
  readonly property int tabs: families.length + 3
  readonly property bool onAchievements: tab === achievementsTab
  readonly property bool onGrid: tab <= achievementsTab
  readonly property var items: onAchievements ? (collection ? collection.achievements : [])
    : (onGrid && families.length ? families[tab].items : [])
  readonly property Flickable view: tab === statsTab ? statsView : tab === historyTab ? historyView : grid

  function move(step) { tab = (tab + step + tabs) % tabs; view.contentY = 0 }
  function scroll(step) {
    view.contentY = Math.max(0, Math.min(view.contentHeight - view.height,
                                         view.contentY + step * Style.space(120)))
  }
  // 33% in English, 33 % in French.
  function percent(part, whole) {
    return whole ? Math.round(100 * part / whole) + (I18n.lang === "fr" ? "\u00a0%" : "%") : "–"
  }
  function deckName(deck) { return deck === null ? I18n.tr("Plain deck") : deck }

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
            readonly property var family: index < root.families.length ? root.families[index] : null
            readonly property string label: family ? family.name + "  " + family.found + "/" + family.total
              : index === root.achievementsTab ? I18n.tr("Achievements") + "  "
                + root.collection.achievements.filter(function(a) { return a.done }).length
                + "/" + root.collection.achievements.length
              : index === root.statsTab ? I18n.tr("Stats")
              : I18n.tr("History") + "  " + root.history.length
            width: tabLabel.implicitWidth + 2 * Style.spacing.lg
            height: tabLabel.implicitHeight + 2 * Style.spacing.sm
            radius: Style.cornerRadius
            color: chosen ? Qt.tint(Colors.background, Util.alpha(Colors.accent, 0.15)) : Style.normalFill
            border.width: chosen ? Style.space(2) : Style.normalBorderWidth
            border.color: chosen ? Colors.accent : Style.normalBorderColor
            Text {
              id: tabLabel
              anchors.centerIn: parent
              text: label
              color: chosen ? Colors.accent : Colors.foreground
              font.family: Style.font.family
              font.pixelSize: Style.font.caption
            }
            MouseArea {
              anchors.fill: parent
              cursorShape: Qt.PointingHandCursor
              onClicked: { root.tab = index; root.view.contentY = 0 }
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
      visible: root.onGrid
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

    // The stats: totals, then per deck and per hand type.
    Flickable {
      id: statsView
      anchors.fill: grid
      visible: root.tab === root.statsTab
      clip: true
      contentHeight: statsColumn.height
      boundsBehavior: Flickable.StopAtBounds

      Column {
        id: statsColumn
        width: parent.width
        spacing: Style.spacing.xl
        readonly property var s: root.stats

        Header {
          visible: !statsColumn.s || statsColumn.s.games + statsColumn.s.hands_played === 0
          text: I18n.tr("No game yet.")
        }

        Flow {
          width: parent.width
          spacing: Style.spacing.md
          visible: statsColumn.s !== null
          Repeater {
            model: statsColumn.s ? [
              [I18n.tr("Games"), Labels.number(statsColumn.s.games)],
              [I18n.tr("Won"), Labels.number(statsColumn.s.won) + " (" + root.percent(statsColumn.s.won, statsColumn.s.games) + ")"],
              [I18n.tr("Lost"), Labels.number(statsColumn.s.lost)],
              [I18n.tr("Abandoned"), Labels.number(statsColumn.s.abandoned)],
              [I18n.tr("Win streak (now / best)"), Labels.number(statsColumn.s.streak) + " / " + Labels.number(statsColumn.s.best_streak)],
              [I18n.tr("Best ante"), Labels.number(statsColumn.s.best_ante)],
              [I18n.tr("Best hand"), Labels.number(statsColumn.s.best_hand)],
              [I18n.tr("Blinds won"), Labels.number(statsColumn.s.blinds_won)],
              [I18n.tr("Hands played"), Labels.number(statsColumn.s.hands_played)],
              [I18n.tr("Jokers sold"), Labels.number(statsColumn.s.jokers_sold)]
            ] : []
            Rectangle {
              width: Style.space(190)
              height: tileColumn.implicitHeight + 2 * Style.spacing.lg
              radius: Style.cornerRadius
              color: Style.normalFill
              border.width: Style.normalBorderWidth
              border.color: Style.normalBorderColor
              Column {
                id: tileColumn
                anchors { left: parent.left; right: parent.right; verticalCenter: parent.verticalCenter; margins: Style.spacing.lg }
                Text {
                  width: parent.width
                  elide: Text.ElideRight
                  text: modelData[1]
                  color: Colors.accent
                  font.family: Style.font.family
                  font.pixelSize: Style.font.title
                  font.bold: true
                  // French thousands spaces come from a taller fallback font.
                  lineHeightMode: Text.FixedHeight
                  lineHeight: Math.ceil(font.pixelSize * 1.3)
                }
                Header { width: parent.width; elide: Text.ElideRight; text: modelData[0] }
              }
            }
          }
        }

        // Per deck: a table, most played first.
        Column {
          width: parent.width
          spacing: Style.spacing.sm
          visible: statsColumn.s !== null && statsColumn.s.decks.length > 0
          readonly property var widths: [Style.space(220), Style.space(90), Style.space(110), Style.space(110), Style.space(160)]
          Text {
            text: I18n.tr("Decks")
            color: Colors.accent
            font.family: Style.font.family
            font.pixelSize: Style.font.subtitle
            font.bold: true
          }
          Repeater {
            model: statsColumn.s ? [null].concat(statsColumn.s.decks) : []
            Row {
              readonly property bool head: modelData === null
              readonly property var cells: head
                ? [I18n.tr("Deck"), I18n.tr("Games"), I18n.tr("Won"), I18n.tr("Best ante"), I18n.tr("Best level")]
                : [root.deckName(modelData.deck), Labels.number(modelData.games),
                   Labels.number(modelData.won) + " (" + root.percent(modelData.won, modelData.games) + ")",
                   Labels.number(modelData.best_ante), modelData.best_stake || "–"]
              Repeater {
                model: parent.cells
                Text {
                  width: parent.parent.widths[index]
                  elide: Text.ElideRight
                  text: modelData
                  color: parent.head ? Colors.muted : Colors.foreground
                  font.family: Style.font.family
                  font.pixelSize: Style.font.caption
                }
              }
            }
          }
        }

        // Per hand type: how often, as a bar against the most played.
        Column {
          width: parent.width
          spacing: Style.spacing.sm
          visible: statsColumn.s !== null && statsColumn.s.hands.length > 0
          Text {
            text: I18n.tr("Hand types")
            color: Colors.accent
            font.family: Style.font.family
            font.pixelSize: Style.font.subtitle
            font.bold: true
          }
          Repeater {
            model: statsColumn.s ? statsColumn.s.hands : []
            Row {
              spacing: Style.spacing.md
              Text {
                width: Style.space(220)
                elide: Text.ElideRight
                text: modelData.hand_name
                color: Colors.foreground
                font.family: Style.font.family
                font.pixelSize: Style.font.caption
              }
              Text {
                width: Style.space(90)
                horizontalAlignment: Text.AlignRight
                text: Labels.number(modelData.played)
                color: Colors.foreground
                font.family: Style.font.family
                font.pixelSize: Style.font.caption
              }
              Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: Math.max(Style.space(2), Style.space(300) * modelData.played / statsColumn.s.hands[0].played)
                height: Style.space(8)
                radius: height / 2
                color: Util.alpha(Colors.accent, 0.6)
              }
            }
          }
        }
      }
    }

    // The history: the latest games, the last one first.
    ListView {
      id: historyView
      anchors.fill: grid
      visible: root.tab === root.historyTab
      clip: true
      spacing: Style.spacing.md
      boundsBehavior: Flickable.StopAtBounds
      model: root.history

      Header {
        visible: root.history.length === 0
        text: I18n.tr("No game yet.")
      }

      delegate: Rectangle {
        readonly property var g: modelData
        readonly property color tint: g.outcome === "won" ? Colors.accent
          : g.outcome === "lost" ? Colors.urgent : Colors.muted
        width: historyView.width
        height: gameColumn.implicitHeight + 2 * Style.spacing.lg
        radius: Style.cornerRadius
        color: Style.normalFill
        border.width: Style.normalBorderWidth
        border.color: Util.alpha(tint, 0.6)

        Column {
          id: gameColumn
          anchors { left: parent.left; right: parent.right; verticalCenter: parent.verticalCenter; margins: Style.spacing.lg }
          spacing: Style.spacing.xs
          Row {
            width: parent.width
            spacing: Style.spacing.lg
            Text {
              text: g.outcome === "won" ? I18n.tr("Victory") : g.outcome === "lost" ? I18n.tr("Defeat") : I18n.tr("Left")
              color: tint
              font.family: Style.font.family
              font.pixelSize: Style.font.body
              font.bold: true
            }
            Text {
              text: I18n.tr("Ante {n}", { n: g.ante }) + " · " + root.deckName(g.deck) + " · " + g.stake
              color: Colors.foreground
              font.family: Style.font.family
              font.pixelSize: Style.font.body
            }
            Header {
              text: Qt.formatDateTime(new Date(g.ended * 1000), "yyyy-MM-dd hh:mm") + " · " + I18n.tr("seed {seed}", { seed: g.seed_code })
            }
          }
          Text {
            width: parent.width
            elide: Text.ElideRight
            text: I18n.tr("Best hand: {score}", { score: Labels.number(g.best_hand) })
              + (g.hand_name ? " · " + I18n.tr("Most played: {hand}", { hand: g.hand_name }) : "")
              + (g.boss ? " · " + I18n.tr("Ended by {boss}", { boss: g.boss }) : "")
            color: Colors.foreground
            font.family: Style.font.family
            font.pixelSize: Style.font.caption
          }
          Header {
            width: parent.width
            elide: Text.ElideRight
            text: g.jokers.length ? g.jokers.join(", ") : I18n.tr("No joker")
          }
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
