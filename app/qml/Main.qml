import QtQuick
import QtQuick.Layouts
import omatro
import "Labels.js" as Labels

// Fenêtre de jeu. Les règles vivent dans le moteur Rust, appelé directement
// (objet Engine) avec le protocole JSON du moteur. Ici, on ne fait qu'afficher
// et animer.
Window {
  id: root

  title: "omatro"
  visible: true
  color: Color.background
  width: Style.space(1100)
  height: Style.space(800)
  minimumWidth: Style.space(980)
  minimumHeight: Style.space(720)

  // ---- moteur et thème ----------------------------------------------------
  Engine { id: engine }

  function send(command) {
    receive(engine.handle(JSON.stringify(command)))
  }

  function receive(line) {
    var r
    try { r = JSON.parse(line) } catch (e) { notice = "Réponse illisible du moteur."; return }
    if (r.event === "state") applyState(r.state)
    else if (r.event === "preview") showPreview(r)
    else if (r.event === "scored") startScoring(r)
    else if (r.event === "error") notice = r.message
  }

  // Changement de thème Omarchy : on relit ses fichiers s'ils ont bougé.
  Timer {
    interval: 1000
    running: true
    repeat: true
    onTriggered: OmarchyTheme.refresh()
  }

  Component.onCompleted: {
    newGame()
    keyCatcher.forceActiveFocus()
  }

  // ---- état de la partie --------------------------------------------------
  property var game: null        // dernier état envoyé par le moteur
  property var hand: []          // cartes affichées dans la main
  property var selected: []      // indices dans la main, dans l'ordre des clics
  property var preview: null
  property var scoring: null     // détail du score en cours d'animation
  property var pendingState: null
  property int stepIndex: -1
  property bool showTotal: false
  property real liveChips: 0
  property real liveMult: 0
  property real shownScore: 0
  property string notice: ""
  Behavior on shownScore { NumberAnimation { duration: 600; easing.type: Easing.OutCubic } }

  readonly property bool busy: scoring !== null
  readonly property bool playing: game !== null && game.phase === "playing" && !busy
  readonly property bool shopping: game !== null && game.phase === "shop" && !busy
  readonly property string handName: scoring ? Labels.hand(scoring.hand)
    : (preview && preview.hand ? Labels.hand(preview.hand) : "")

  function newGame() {
    notice = ""
    send({ cmd: "new_game" })
  }

  function applyState(s) {
    game = s
    hand = s.hand
    selected = []
    preview = null
    liveChips = 0
    liveMult = 0
    shownScore = s.score
  }

  function toggleCard(i) {
    if (!playing || i < 0 || i >= hand.length) return
    var s = selected.slice()
    var k = s.indexOf(i)
    if (k >= 0) s.splice(k, 1)
    else if (s.length < 5) s.push(i)
    else return
    selected = s
    notice = ""
    send({ cmd: "preview", cards: s })
  }

  function showPreview(r) {
    if (JSON.stringify(r.cards) !== JSON.stringify(selected)) return
    preview = r
    liveChips = r.base_chips
    liveMult = r.base_mult
  }

  function play() {
    if (!playing || selected.length === 0) return
    send({ cmd: "play", cards: selected })
  }

  function discard() {
    if (!playing || selected.length === 0) return
    if (game.discards_left === 0) { notice = "Plus de défausse."; return }
    send({ cmd: "discard", cards: selected })
  }

  // Entrée : jouer, encaisser, quitter la boutique ou recommencer.
  function confirm() {
    if (busy || !game) return
    notice = ""
    if (game.phase === "won") send({ cmd: "cash_out" })
    else if (game.phase === "shop") send({ cmd: "next_blind" })
    else if (game.phase === "lost") newGame()
    else play()
  }

  // ---- boutique -----------------------------------------------------------
  function canBuy(offer) {
    return shopping && game.money >= offer.price && game.jokers.length < game.max_jokers
  }
  function buy(i) {
    if (shopping && i >= 0 && i < game.shop.length) send({ cmd: "buy", offer: i })
  }
  function sell(i) {
    if (shopping) send({ cmd: "sell", joker: i })
  }
  function reroll() {
    if (!shopping) return
    if (game.money < game.reroll_cost) { notice = "Pas assez d'argent pour relancer."; return }
    send({ cmd: "reroll" })
  }

  // ---- animation du score -------------------------------------------------
  // Les cartes jouées passent sur la table, puis chaque étape du détail
  // (carte ou joker) rebondit et met à jour jetons × mult.
  function startScoring(r) {
    var rest = []
    for (var i = 0; i < hand.length; i++)
      if (selected.indexOf(i) < 0) rest.push(hand[i])
    hand = rest
    selected = []
    preview = null
    scoring = r.score
    pendingState = r.state
    stepIndex = -1
    showTotal = false
    liveChips = r.score.base_chips
    liveMult = r.score.base_mult
    stepTimer.interval = 600
    stepTimer.start()
  }

  function nextStep() {
    stepTimer.interval = 420
    var steps = scoring.steps
    stepIndex++
    if (stepIndex < steps.length) {
      var step = steps[stepIndex]
      liveChips = step.chips
      liveMult = step.mult
      var color = Labels.isChips(step.effect) ? Color.accent : Color.urgent
      var view = step.source.kind === "card"
        ? tableRepeater.itemAt(step.source.index)
        : jokerRepeater.itemAt(step.source.index)
      if (view) view.flash(Labels.effect(step.effect), color)
    } else if (stepIndex === steps.length) {
      showTotal = true
      stepTimer.interval = 900
    } else {
      stepTimer.stop()
      notice = Labels.hand(scoring.hand) + " : " + Labels.number(scoring.total) + " points"
      var s = pendingState
      scoring = null
      pendingState = null
      showTotal = false
      applyState(s)
    }
  }

  Timer {
    id: stepTimer
    repeat: true
    onTriggered: root.nextStep()
  }

  // ---- fenêtre ------------------------------------------------------------
  component Box: Rectangle {
    radius: Style.cornerRadius
    color: Style.normalFill
    border.width: Style.normalBorderWidth
    border.color: Style.normalBorderColor
  }

  component Caption: Text {
    color: Color.muted
    font.family: Style.font.family
    font.pixelSize: Style.font.caption
  }

  component Value: Text {
    color: Color.foreground
    font.family: Style.font.family
    font.pixelSize: Style.font.display
    font.bold: true
  }

  Item {
    id: keyCatcher
    anchors.fill: parent
    focus: true

    Keys.onPressed: function(event) {
      var k = event.key
      if (k === Qt.Key_Q && (event.modifiers & Qt.ControlModifier)) Qt.quit()
      else if (k >= Qt.Key_1 && k <= Qt.Key_8) {
        if (root.shopping) root.buy(k - Qt.Key_1)
        else root.toggleCard(k - Qt.Key_1)
      }
      else if (k === Qt.Key_R) root.reroll()
      else if (k === Qt.Key_Return || k === Qt.Key_Enter || k === Qt.Key_Space) root.confirm()
      else if (k === Qt.Key_D || k === Qt.Key_Delete) root.discard()
      else if (k === Qt.Key_N) root.newGame()
      else return
      event.accepted = true
    }

    RowLayout {
      anchors { fill: parent; margins: Style.spacing.huge }
      spacing: Style.spacing.huge

      // ---- colonne de gauche : blind, score, calcul en cours
      // Largeur fixe : un layout imbriqué s'étire par défaut et la colonne
      // changerait de taille selon le contenu de droite (jeu ou boutique).
      ColumnLayout {
        Layout.fillWidth: false
        Layout.preferredWidth: Style.space(250)
        Layout.minimumWidth: Style.space(250)
        Layout.maximumWidth: Style.space(250)
        Layout.fillHeight: true
        spacing: Style.spacing.xxl

        Text {
          text: "omatro"
          color: Color.accent
          font.family: Style.font.family
          font.pixelSize: Style.font.displayLarge
          font.bold: true
        }

        Box {
          Layout.fillWidth: true
          implicitHeight: blindCol.implicitHeight + 2 * Style.spacing.xxl
          Column {
            id: blindCol
            anchors { fill: parent; margins: Style.spacing.xxl }
            spacing: Style.spacing.xs
            Caption { text: root.game ? "Blind " + root.game.blind + " · objectif" : "Blind" }
            Value { text: root.game ? Labels.number(root.game.target) : "–" }
          }
        }

        Box {
          Layout.fillWidth: true
          implicitHeight: scoreCol.implicitHeight + 2 * Style.spacing.xxl
          Column {
            id: scoreCol
            anchors { fill: parent; margins: Style.spacing.xxl }
            spacing: Style.spacing.xs
            Caption { text: "Score de la blind" }
            Value { text: Labels.number(root.shownScore) }
          }
        }

        Box {
          Layout.fillWidth: true
          implicitHeight: moneyCol.implicitHeight + 2 * Style.spacing.xxl
          Column {
            id: moneyCol
            anchors { fill: parent; margins: Style.spacing.xxl }
            spacing: Style.spacing.xs
            Caption { text: "Argent" }
            Value { text: root.game ? Labels.money(root.game.money) : "–"; color: Color.accent }
          }
        }

        // Main reconnue et jetons × mult (aperçu, puis calcul animé).
        Box {
          Layout.fillWidth: true
          implicitHeight: calcCol.implicitHeight + 2 * Style.spacing.xxl
          Column {
            id: calcCol
            anchors { fill: parent; margins: Style.spacing.xxl }
            spacing: Style.spacing.md
            Text {
              text: root.handName || "Choisir des cartes"
              color: root.handName ? Color.foreground : Color.muted
              font.family: Style.font.family
              font.pixelSize: Style.font.heading
              font.bold: root.handName !== ""
            }
            Row {
              spacing: Style.spacing.md
              Rectangle {
                width: Style.space(90); height: Style.space(44)
                radius: Style.cornerRadius
                color: Util.alpha(Color.accent, 0.18)
                border.width: Style.normalBorderWidth
                border.color: Color.accent
                Text {
                  anchors.centerIn: parent
                  text: Labels.number(root.liveChips)
                  color: Color.accent
                  font.family: Style.font.family
                  font.pixelSize: Style.font.heading
                  font.bold: true
                }
              }
              Text {
                anchors.verticalCenter: parent.verticalCenter
                text: "×"
                color: Color.muted
                font.family: Style.font.family
                font.pixelSize: Style.font.heading
              }
              Rectangle {
                width: Style.space(90); height: Style.space(44)
                radius: Style.cornerRadius
                color: Util.alpha(Color.urgent, 0.18)
                border.width: Style.normalBorderWidth
                border.color: Color.urgent
                Text {
                  anchors.centerIn: parent
                  text: Labels.mult(root.liveMult)
                  color: Color.urgent
                  font.family: Style.font.family
                  font.pixelSize: Style.font.heading
                  font.bold: true
                }
              }
            }
          }
        }

        RowLayout {
          Layout.fillWidth: true
          spacing: Style.spacing.xxl
          Box {
            Layout.fillWidth: true
            implicitHeight: handsCol.implicitHeight + 2 * Style.spacing.xl
            Column {
              id: handsCol
              anchors { fill: parent; margins: Style.spacing.xl }
              Caption { text: "Mains" }
              Value { text: root.game ? root.game.hands_left : "–"; color: Color.accent }
            }
          }
          Box {
            Layout.fillWidth: true
            implicitHeight: discardsCol.implicitHeight + 2 * Style.spacing.xl
            Column {
              id: discardsCol
              anchors { fill: parent; margins: Style.spacing.xl }
              Caption { text: "Défausses" }
              Value { text: root.game ? root.game.discards_left : "–"; color: Color.urgent }
            }
          }
        }

        Caption { text: root.game ? "Pioche : " + root.game.deck_left + " / 52" : "" }

        Item { Layout.fillHeight: true }

        Caption {
          Layout.fillWidth: true
          wrapMode: Text.WordWrap
          text: root.shopping
            ? "1, 2 : acheter · R : relancer\nEntrée : blind suivante · Ctrl+Q : quitter"
            : "1 à 8 : choisir · Entrée : jouer\nD : défausser · N : nouvelle partie\nCtrl+Q : quitter"
        }
      }

      // ---- zone de jeu : jokers, table, main
      ColumnLayout {
        Layout.fillWidth: true
        Layout.fillHeight: true
        spacing: Style.spacing.huge

        Caption {
          Layout.alignment: Qt.AlignHCenter
          text: root.game ? "Jokers " + root.game.jokers.length + " / " + root.game.max_jokers : ""
        }

        Row {
          Layout.alignment: Qt.AlignHCenter
          Layout.preferredHeight: Style.space(150) + (root.shopping ? Style.space(48) : 0)
          spacing: Style.spacing.xxl
          Repeater {
            id: jokerRepeater
            model: root.game ? root.game.jokers : []
            Column {
              spacing: Style.spacing.md
              function flash(text, color) { view.flash(text, color) }
              JokerView { id: view; joker: modelData }
              Button {
                anchors.horizontalCenter: parent.horizontalCenter
                visible: root.shopping
                text: "Vendre " + Labels.money(modelData.sell_value)
                bordered: true
                onClicked: root.sell(index)
              }
            }
          }
        }

        // Boutique, entre deux blinds.
        ColumnLayout {
          Layout.fillWidth: true
          Layout.fillHeight: true
          visible: root.game !== null && root.game.phase === "shop"
          spacing: Style.spacing.huge

          Item { Layout.fillHeight: true }

          Text {
            Layout.alignment: Qt.AlignHCenter
            text: "Boutique"
            color: Color.accent
            font.family: Style.font.family
            font.pixelSize: Style.font.displayLarge
            font.bold: true
          }

          Row {
            Layout.alignment: Qt.AlignHCenter
            spacing: Style.space(40)
            Repeater {
              model: root.game ? root.game.shop : []
              Column {
                spacing: Style.spacing.md
                JokerView {
                  joker: modelData
                  opacity: root.canBuy(modelData) ? 1 : 0.5
                }
                Button {
                  anchors.horizontalCenter: parent.horizontalCenter
                  text: (index + 1) + " · " + Labels.money(modelData.price)
                  bordered: true
                  enabled: root.canBuy(modelData)
                  opacity: enabled ? 1 : 0.5
                  onClicked: root.buy(index)
                }
              }
            }
          }

          Caption {
            Layout.alignment: Qt.AlignHCenter
            visible: root.game !== null && root.game.shop.length === 0
            text: "Plus rien à vendre : relancer pour de nouvelles offres."
          }

          Text {
            Layout.alignment: Qt.AlignHCenter
            text: root.notice
            color: Color.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.title
          }

          Item { Layout.fillHeight: true }

          Row {
            Layout.alignment: Qt.AlignHCenter
            spacing: Style.spacing.xxl
            Button {
              text: "Relancer (R) · " + (root.game ? Labels.money(root.game.reroll_cost) : "")
              bordered: true
              enabled: root.shopping && root.game.money >= root.game.reroll_cost
              opacity: enabled ? 1 : 0.5
              onClicked: root.reroll()
            }
            Button {
              text: "Blind suivante (Entrée)"
              bordered: true
              enabled: root.shopping
              onClicked: root.confirm()
            }
          }
        }

        // Table : cartes en train de marquer, ou dernier résultat.
        Item {
          Layout.fillWidth: true
          Layout.fillHeight: true
          visible: !root.shopping

          Row {
            anchors.centerIn: parent
            spacing: Style.spacing.xxl
            visible: root.scoring !== null
            Repeater {
              id: tableRepeater
              model: root.scoring ? root.scoring.cards : []
              CardView {
                card: modelData
                interactive: false
                dimmed: root.scoring !== null && root.scoring.scoring.indexOf(index) < 0
              }
            }
          }

          Text {
            anchors { horizontalCenter: parent.horizontalCenter; bottom: parent.bottom }
            visible: root.showTotal
            text: root.scoring ? "= " + Labels.number(root.scoring.total) : ""
            color: Color.foreground
            font.family: Style.font.family
            font.pixelSize: Style.font.displayLarge
            font.bold: true
          }

          Text {
            anchors.centerIn: parent
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            visible: root.scoring === null
            text: root.notice
            color: Color.muted
            font.family: Style.font.family
            font.pixelSize: Style.font.title
          }
        }

        Row {
          Layout.alignment: Qt.AlignHCenter
          Layout.topMargin: Style.space(24)
          visible: !root.shopping
          spacing: Style.spacing.lg
          Repeater {
            model: root.hand
            CardView {
              card: modelData
              selected: root.selected.indexOf(index) >= 0
              interactive: root.playing
              shortcut: String(index + 1)
              onClicked: root.toggleCard(index)
            }
          }
        }

        Row {
          Layout.alignment: Qt.AlignHCenter
          Layout.topMargin: Style.space(16)
          visible: !root.shopping
          spacing: Style.spacing.xxl
          Button {
            text: "Jouer la main"
            bordered: true
            enabled: root.playing && root.selected.length > 0
            opacity: enabled ? 1 : 0.5
            onClicked: root.play()
          }
          Button {
            text: "Défausser"
            bordered: true
            enabled: root.playing && root.selected.length > 0 && root.game.discards_left > 0
            opacity: enabled ? 1 : 0.5
            onClicked: root.discard()
          }
        }
      }
    }

    // ---- fin de blind : gagnée ou perdue
    Rectangle {
      anchors.fill: parent
      visible: root.game !== null && (root.game.phase === "won" || root.game.phase === "lost") && !root.busy
      color: Util.alpha(Color.background, 0.85)

      MouseArea { anchors.fill: parent }

      Box {
        anchors.centerIn: parent
        width: Style.space(420)
        height: endCol.implicitHeight + 2 * Style.spacing.huge
        color: Color.background
        border.color: root.game && root.game.phase === "won" ? Color.accent : Color.urgent

        Column {
          id: endCol
          anchors { fill: parent; margins: Style.spacing.huge }
          spacing: Style.spacing.xxl
          readonly property bool won: root.game !== null && root.game.phase === "won"

          Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: endCol.won ? "Blind gagnée !" : "Partie perdue"
            color: endCol.won ? Color.accent : Color.urgent
            font.family: Style.font.family
            font.pixelSize: Style.font.displayLarge
            font.bold: true
          }
          Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: root.game ? Labels.number(root.game.score) + " / " + Labels.number(root.game.target)
              + (endCol.won ? "" : " · blind " + root.game.blind) : ""
            color: Color.foreground
            font.family: Style.font.family
            font.pixelSize: Style.font.title
          }

          // Détail de la récompense.
          Grid {
            anchors.horizontalCenter: parent.horizontalCenter
            visible: endCol.won && root.game.reward !== null
            columns: 2
            columnSpacing: Style.space(40)
            rowSpacing: Style.spacing.sm
            Repeater {
              model: {
                var r = root.game && root.game.reward
                if (!r) return []
                var lines = [["Blind", r.blind], ["Mains restantes", r.hands], ["Intérêts", r.interest]]
                if (r.jokers > 0) lines.push(["Jokers", r.jokers])
                lines.push(["Total", r.total])
                var cells = []
                for (var i = 0; i < lines.length; i++)
                  cells.push({ text: lines[i][0], value: false, last: i === lines.length - 1 },
                             { text: "+" + Labels.money(lines[i][1]), value: true, last: i === lines.length - 1 })
                return cells
              }
              Text {
                text: modelData.text
                color: modelData.value ? Color.accent : (modelData.last ? Color.foreground : Color.muted)
                font.family: Style.font.family
                font.pixelSize: Style.font.subtitle
                font.bold: modelData.last
              }
            }
          }

          Button {
            anchors.horizontalCenter: parent.horizontalCenter
            text: endCol.won ? "Encaisser (Entrée)" : "Nouvelle partie (Entrée)"
            bordered: true
            onClicked: root.confirm()
          }
        }
      }
    }
  }
}
