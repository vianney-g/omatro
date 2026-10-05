import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Ui
import "Labels.js" as Labels

// Fenêtre de jeu. Les règles vivent dans le moteur Rust (bin/omatro-engine),
// lancé une fois pour toutes : on lui écrit une commande JSON par ligne et il
// répond de même. Ici, on ne fait qu'afficher et animer.
//
//   omarchy-shell shell toggle io.github.vianney-g.omatro '{}'
Item {
  id: root

  // ---- cycle de vie du panneau --------------------------------------------
  property var shell: null
  property var manifest: null
  property bool opened: false
  property bool closingFromHost: false
  readonly property string pluginId: manifest && manifest.id ? String(manifest.id) : "io.github.vianney-g.omatro"

  function open(payloadJson) {
    closingFromHost = false
    opened = true
    window.visible = true
    Qt.callLater(function() { keyCatcher.forceActiveFocus() })
  }

  // Fermeture demandée par le shell : il est déjà au courant.
  function close() {
    closingFromHost = true
    opened = false
    window.visible = false
    closingFromHost = false
  }

  // Fermeture par le joueur (Échap, fermeture de la fenêtre) : on prévient le
  // shell pour que le prochain toggle rouvre bien.
  function requestClose() {
    if (shell && typeof shell.hide === "function") shell.hide(pluginId)
    else close()
  }

  // ---- moteur -------------------------------------------------------------
  readonly property string enginePath: decodeURIComponent(String(Qt.resolvedUrl("bin/omatro-engine")).replace(/^file:\/\//, ""))
  property bool engineReady: false
  property string engineError: ""

  function send(command) {
    if (!engine.running) return
    waiting = true
    engine.write(JSON.stringify(command) + "\n")
  }

  function receive(line) {
    waiting = false
    var r
    try { r = JSON.parse(line) } catch (e) { notice = "Réponse illisible du moteur."; return }
    engineReady = true
    engineError = ""
    if (r.event === "state") applyState(r.state)
    else if (r.event === "preview") showPreview(r)
    else if (r.event === "scored") startScoring(r)
    else if (r.event === "error") notice = r.message
  }

  Process {
    id: engine
    command: [root.enginePath]
    running: true
    stdinEnabled: true
    stdout: SplitParser { onRead: data => root.receive(data) }
    onStarted: root.newGame()
    onExited: function(code) {
      root.engineReady = false
      root.waiting = false
      root.engineError = "Le moteur s'est arrêté (code " + code + "). N pour le relancer."
    }
  }

  // Pas de réponse au démarrage : le binaire manque sans doute.
  Timer {
    interval: 1500
    running: true
    onTriggered: if (!root.engineReady && root.engineError === "")
      root.engineError = "Moteur introuvable : " + root.enginePath + "\nLancer « make plugin » à la racine du dépôt."
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
  property bool waiting: false
  property real liveChips: 0
  property real liveMult: 0
  property real shownScore: 0
  property string notice: ""
  Behavior on shownScore { NumberAnimation { duration: 600; easing.type: Easing.OutCubic } }

  readonly property bool busy: waiting || scoring !== null
  readonly property bool playing: game !== null && game.phase === "playing" && !busy
  readonly property string handName: scoring ? Labels.hand(scoring.hand)
    : (preview && preview.hand ? Labels.hand(preview.hand) : "")

  // Tous les jokers en attendant la boutique.
  function newGame() {
    if (!engine.running) {
      engineError = ""
      engine.running = true
      return
    }
    notice = ""
    send({ cmd: "new_game", jokers: ["baobab", "marche", "griot", "harmattan"] })
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

  // Entrée : jouer, passer à la blind suivante ou recommencer.
  function confirm() {
    if (busy || !game) return
    if (game.phase === "won") send({ cmd: "next_blind" })
    else if (game.phase === "lost") newGame()
    else play()
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

  FloatingWindow {
    id: window
    title: "omatro"
    color: Color.background
    implicitWidth: Style.space(1100)
    implicitHeight: Style.space(740)
    minimumSize: Qt.size(Style.space(980), Style.space(660))
    visible: false

    // Fermée par Hyprland (SUPER + W…).
    onVisibleChanged: if (!visible && root.opened && !root.closingFromHost) root.requestClose()

    Item {
      id: keyCatcher
      anchors.fill: parent
      focus: true

      Keys.onPressed: function(event) {
        var k = event.key
        if (k === Qt.Key_Escape) root.requestClose()
        else if (k >= Qt.Key_1 && k <= Qt.Key_8) root.toggleCard(k - Qt.Key_1)
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
        ColumnLayout {
          Layout.preferredWidth: Style.space(250)
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
            text: "1 à 8 : choisir · Entrée : jouer\nD : défausser · N : nouvelle partie\nÉchap : fermer"
          }
        }

        // ---- zone de jeu : jokers, table, main
        ColumnLayout {
          Layout.fillWidth: true
          Layout.fillHeight: true
          spacing: Style.spacing.huge

          Row {
            Layout.alignment: Qt.AlignHCenter
            spacing: Style.spacing.xxl
            Repeater {
              id: jokerRepeater
              model: root.game ? root.game.jokers : []
              JokerView { joker: modelData }
            }
          }

          // Table : cartes en train de marquer, ou dernier résultat.
          Item {
            Layout.fillWidth: true
            Layout.fillHeight: true

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
              text: root.engineError || root.notice
              color: root.engineError ? Color.urgent : Color.muted
              font.family: Style.font.family
              font.pixelSize: Style.font.title
            }
          }

          Row {
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: Style.space(24)
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
        visible: root.game !== null && root.game.phase !== "playing" && !root.busy
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
            Button {
              anchors.horizontalCenter: parent.horizontalCenter
              text: endCol.won ? "Blind suivante (Entrée)" : "Nouvelle partie (Entrée)"
              bordered: true
              onClicked: root.confirm()
            }
          }
        }
      }
    }
  }
}
