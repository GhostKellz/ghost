import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects
import QtQuick.Shapes

// Ghost SDDM ghostTheme. SDDM provides the context objects `sddm`, `userModel`,
// `sessionModel`, `keyboard` and `config`; authentication and session launch
// happen only through sddm.login().
Item {
    id: root
    width: 1920
    height: 1080

    function cfg(key, fallback) {
        var value = config[key]
        return (value === undefined || value === null || value === "") ? fallback : value
    }

    // Relative paths resolve against this theme directory, never the repository.
    function resolvePath(path) {
        if (!path)
            return ""
        return path.charAt(0) === "/" ? "file://" + path : Qt.resolvedUrl(path)
    }

    QtObject {
        id: ghostTheme
        // Layout is designed at 1080p and scales with the screen, so it keeps
        // its proportions at 4K whether or not Qt high-DPI scaling is active.
        readonly property real s: Math.max(0.85, Math.min(root.width / 1920, root.height / 1080))
        readonly property color background: root.cfg("backgroundColor", "#1a1b26")
        readonly property color surface: "#16161e"
        // Fields are tinted glass over the panel; the session popup stays opaque for legibility.
        readonly property color field: Qt.rgba(0.122, 0.137, 0.208, 0.62)
        readonly property color fieldSolid: "#1f2335"
        readonly property color border: "#414868"
        readonly property color text: "#c0caf5"
        readonly property color textSoft: "#a9b1d6"
        readonly property color accent: root.cfg("accent", "#7aa2f7")
        readonly property color accentAlt: root.cfg("accentAlt", "#7dcfff")
        readonly property color error: "#f7768e"
        readonly property string font: root.cfg("font", "Noto Sans")
    }

    // SDDM loads this file once per screen; only the primary one gets the panel.
    readonly property bool showPanel: typeof primaryScreen === "undefined" || primaryScreen
    readonly property real panelOpacity: Number(cfg("panelOpacity", 0.55))
    readonly property real blurRadius: Number(cfg("blur", 64))
    readonly property bool manualUser: userModel.count === 0
    property int userIndex: Math.max(0, Math.min(userModel.lastIndex, userModel.count - 1))
    readonly property var currentUser:userRepeater.count > 0 ? userRepeater.itemAt(userIndex) : null
    readonly property string userName: manualUser ? usernameField.text.trim() : (currentUser ? currentUser.name : "")
    property bool busy: false
    property string pendingAction: ""   // "poweroff" | "reboot" while the confirmation is open
    property string message: ""
    property bool messageIsError: false

    function login() {
        if (busy)
            return
        if (userName === "") {
            showMessage("Enter a username.", true)
            usernameField.forceActiveFocus()
            return
        }
        busy = true
        message = ""
        sddm.login(userName, password.text, session.currentIndex)
    }

    function showMessage(text, isError) {
        message = text
        messageIsError = isError
    }

    function focusCredentials() {
        if (manualUser && usernameField.text === "")
            usernameField.forceActiveFocus()
        else
            password.forceActiveFocus()
    }

    function selectUser(step) {
        userIndex = (userIndex + step + userModel.count) % userModel.count
        password.clear()
        message = ""
        password.forceActiveFocus()
    }

    function requestPower(action) {
        pendingAction = action
        cancelButton.forceActiveFocus()
    }

    function closeConfirm() {
        var action = pendingAction
        pendingAction = ""
        if (action === "poweroff")
            powerOffButton.forceActiveFocus()
        else if (action === "reboot")
            rebootButton.forceActiveFocus()
    }

    function confirmPower() {
        var action = pendingAction
        pendingAction = ""
        if (action === "poweroff")
            sddm.powerOff()
        else if (action === "reboot")
            sddm.reboot()
    }

    Connections {
        target: sddm
        function onLoginFailed() {
            root.busy = false
            password.clear()
            root.showMessage("Login failed. Check your password and try again.", true)
            shake.restart()
            root.focusCredentials()
        }
        function onLoginSucceeded() {
            root.busy = false
        }
        function onInformationMessage(message) {
            root.showMessage(message, false)
        }
    }

    Component.onCompleted: if (showPanel) focusCredentials()

    // Role data for the user list; SDDM exposes users only as a model.
    Item {
        visible: false
        Repeater {
            id: userRepeater
            model: userModel
            delegate: Item {
                required property string name
                required property string realName
                required property string icon
            }
        }
    }

    Item {
        id: backdrop
        anchors.fill: parent

        Rectangle {
            anchors.fill: parent
            color: ghostTheme.background
        }

        // Default backdrop when no wallpaper is set: soft blue and cyan light
        // over the base colour, so the frosted panel has something to blur.
        // Drawn with Canvas, which also works under software rendering.
        Canvas {
            id: glow
            objectName: "glow"
            anchors.fill: parent
            visible: !wallpaper.visible && root.cfg("glow", "true") !== "false"
            onWidthChanged: requestPaint()
            onHeightChanged: requestPaint()
            onPaint: {
                var ctx = getContext("2d")
                var w = width, h = height
                ctx.reset()
                var base = ctx.createLinearGradient(0, 0, 0, h)
                base.addColorStop(0, "#13141c")
                base.addColorStop(1, "#1d2030")
                ctx.fillStyle = base
                ctx.fillRect(0, 0, w, h)
                // [x, y, radius] as fractions of the screen, colour, peak alpha.
                var lights = [
                    [0.30, 0.36, 0.42, "122,162,247", 0.42],   // blue, behind the panel's left half
                    [0.72, 0.66, 0.36, "125,207,255", 0.30],   // cyan, behind the lower right
                    [0.86, 0.14, 0.26, "61,89,161", 0.55],     // deep blue, top right
                    [0.10, 0.88, 0.30, "61,89,161", 0.45],     // deep blue, bottom left
                ]
                for (var i = 0; i < lights.length; ++i) {
                    var l = lights[i]
                    var r = l[2] * Math.max(w, h)
                    var g = ctx.createRadialGradient(l[0] * w, l[1] * h, 0, l[0] * w, l[1] * h, r)
                    g.addColorStop(0, "rgba(" + l[3] + "," + l[4] + ")")
                    g.addColorStop(0.45, "rgba(" + l[3] + "," + (l[4] * 0.35) + ")")
                    g.addColorStop(1, "rgba(" + l[3] + ",0)")
                    ctx.fillStyle = g
                    ctx.fillRect(0, 0, w, h)
                }
            }
        }

        Image {
            id: wallpaper
            objectName: "wallpaper"
            anchors.fill: parent
            source: root.resolvePath(root.cfg("background", ""))
            fillMode: Image.PreserveAspectCrop
            asynchronous: true
            visible: status === Image.Ready
        }
    }

    // Frosted glass: blur only the part of the backdrop behind the panel,
    // clipped to the panel's rounded shape.
    ShaderEffectSource {
        id: behindPanel
        sourceItem: backdrop
        sourceRect: Qt.rect(panel.x, panel.y, panel.width, panel.height)
        visible: false
    }

    Rectangle {
        id: panelMask
        width: panel.width
        height: panel.height
        radius: panel.radius
        visible: false
        layer.enabled: true
    }

    MultiEffect {
        anchors.fill: panel
        source: behindPanel
        visible: root.showPanel && (wallpaper.visible || glow.visible) && root.blurRadius > 0
        autoPaddingEnabled: false
        blurEnabled: true
        blur: 1.0
        blurMax: root.blurRadius
        blurMultiplier: 1.0
        // A little extra saturation keeps colours vivid through the frosting.
        saturation: 0.2
        maskEnabled: true
        maskSource: panelMask
        maskThresholdMin: 0.5
        maskSpreadAtMin: 1.0
    }

    // Soft drop shadow lifts the glass off the background.
    RectangularShadow {
        anchors.fill: panel
        visible: root.showPanel
        radius: panel.radius
        offset.y: 14 * ghostTheme.s
        blur: 40 * ghostTheme.s
        color: Qt.rgba(0, 0, 0, 0.45)
        z: -1
    }

    Rectangle {
        id: panel
        objectName: "panel"
        visible: root.showPanel
        anchors.centerIn: parent
        width: Math.min(760 * ghostTheme.s, root.width - 32)
        height: Math.min(440 * ghostTheme.s, root.height - 32)
        radius: 22 * ghostTheme.s
        // Slightly clearer at the top than the bottom, like light falling on glass.
        gradient: Gradient {
            GradientStop {
                position: 0
                color: Qt.rgba(ghostTheme.surface.r, ghostTheme.surface.g, ghostTheme.surface.b, Math.max(0, root.panelOpacity - 0.08))
            }
            GradientStop {
                position: 1
                color: Qt.rgba(ghostTheme.surface.r, ghostTheme.surface.g, ghostTheme.surface.b, Math.min(1, root.panelOpacity + 0.08))
            }
        }
        border.width: Math.max(1, Math.round(ghostTheme.s))
        border.color: Qt.rgba(ghostTheme.accent.r, ghostTheme.accent.g, ghostTheme.accent.b, 0.28)

        // Faint sheen over the upper half and a brighter top edge.
        Rectangle {
            width: parent.width
            height: parent.height * 0.5
            radius: parent.radius
            gradient: Gradient {
                GradientStop { position: 0; color: Qt.rgba(1, 1, 1, 0.07) }
                GradientStop { position: 1; color: Qt.rgba(1, 1, 1, 0) }
            }
        }
        Rectangle {
            x: parent.radius
            y: Math.max(1, Math.round(ghostTheme.s))
            width: parent.width - 2 * parent.radius
            height: Math.max(1, Math.round(ghostTheme.s))
            color: Qt.rgba(1, 1, 1, 0.14)
        }

        // Sidebar fill: a wider rounded rectangle clipped to the sidebar, so the
        // left corners follow the panel and the right edge is square.
        Item {
            id: sidebar
            width: 84 * ghostTheme.s
            height: parent.height
            clip: true

            Rectangle {
                width: parent.width + panel.radius
                height: parent.height
                radius: panel.radius
                color: Qt.rgba(0, 0, 0, 0.22)
            }
        }

        Rectangle {
            x: sidebar.width
            width: Math.max(1, Math.round(ghostTheme.s))
            height: parent.height
            color: Qt.rgba(ghostTheme.accent.r, ghostTheme.accent.g, ghostTheme.accent.b, 0.14)
        }

        // Main area is declared before the sidebar buttons so Tab reaches the
        // credentials first, then the session selector, then power controls.
        Item {
            id: mainArea
            x: sidebar.width
            width: parent.width - sidebar.width
            height: parent.height
            enabled: root.pendingAction === ""
            opacity: enabled ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: 120 } }

            Column {
                anchors.centerIn: parent
                anchors.verticalCenterOffset: -18 * ghostTheme.s
                spacing: 0

                Row {
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: 18 * ghostTheme.s

                    IconButton {
                        objectName: "previousUser"
                        theme: ghostTheme
                        anchors.verticalCenter: parent.verticalCenter
                        visible: userModel.count > 1
                        label: "Previous user"
                        iconPath: "M15 6l-6 6l6 6"
                        onActivated: root.selectUser(-1)
                    }

                    // Avatar: gradient ring around a circular image.
                    Rectangle {
                        id: avatarRing
                        width: 116 * ghostTheme.s
                        height: width
                        radius: width / 2
                        gradient: Gradient {
                            orientation: Gradient.Horizontal
                            GradientStop { position: 0; color: ghostTheme.accent }
                            GradientStop { position: 1; color: ghostTheme.accentAlt }
                        }

                        Rectangle {
                            anchors.fill: parent
                            anchors.margins: 3 * ghostTheme.s
                            radius: width / 2
                            color: ghostTheme.surface
                        }

                        Image {
                            id: avatar
                            objectName: "avatar"
                            anchors.fill: parent
                            anchors.margins: 7 * ghostTheme.s
                            source: {
                                // SDDM passes a file:// URL and substitutes its generic
                                // faces/.face.icon when the user has no picture.
                                var icon = root.currentUser ? String(root.currentUser.icon) : ""
                                var hasPicture = icon !== "" && !icon.endsWith("/sddm/faces/.face.icon")
                                if (root.cfg("avatar", "logo") === "user" && hasPicture)
                                    return icon
                                return root.resolvePath(root.cfg("logo", "assets/logo.png"))
                            }
                            visible: false
                        }

                        // Circular crop drawn with Canvas rather than a shader mask, so
                        // the avatar also renders under Qt's software scene graph.
                        Canvas {
                            id: avatarCanvas
                            readonly property url image: avatar.source
                            anchors.fill: avatar
                            visible: avatar.status === Image.Ready
                            onImageChanged: if (image != "") loadImage(image)
                            onImageLoaded: requestPaint()
                            onWidthChanged: requestPaint()
                            onPaint: {
                                var ctx = getContext("2d")
                                ctx.reset()
                                if (!isImageLoaded(image))
                                    return
                                // Center-crop to a square; the hidden Image reports the file's size.
                                var w = avatar.implicitWidth
                                var h = avatar.implicitHeight
                                var side = Math.min(w, h)
                                ctx.beginPath()
                                ctx.arc(width / 2, height / 2, width / 2, 0, 2 * Math.PI)
                                ctx.closePath()
                                ctx.clip()
                                ctx.drawImage(image, (w - side) / 2, (h - side) / 2, side, side, 0, 0, width, height)
                            }
                        }

                        // Fallback when no image loads: the user's initial.
                        Text {
                            anchors.centerIn: parent
                            visible: avatar.status !== Image.Ready
                            text: root.userName !== "" ? root.userName.charAt(0).toUpperCase() : "?"
                            color: ghostTheme.accent
                            font.family: ghostTheme.font
                            font.pixelSize: 44 * ghostTheme.s
                            font.weight: Font.DemiBold
                        }
                    }

                    IconButton {
                        objectName: "nextUser"
                        theme: ghostTheme
                        anchors.verticalCenter: parent.verticalCenter
                        visible: userModel.count > 1
                        label: "Next user"
                        iconPath: "M9 6l6 6l-6 6"
                        onActivated: root.selectUser(1)
                    }
                }

                Item { width: 1; height: 14 * ghostTheme.s }

                Text {
                    objectName: "userLabel"
                    anchors.horizontalCenter: parent.horizontalCenter
                    visible: !root.manualUser
                    text: root.currentUser ? (root.currentUser.realName || root.currentUser.name) : ""
                    color: ghostTheme.text
                    font.family: ghostTheme.font
                    font.pixelSize: 21 * ghostTheme.s
                    font.weight: Font.Medium
                }

                Item { width: 1; height: 22 * ghostTheme.s; visible: !root.manualUser }

                TextField {
                    id: usernameField
                    objectName: "usernameField"
                    anchors.horizontalCenter: parent.horizontalCenter
                    visible: root.manualUser
                    width: 340 * ghostTheme.s
                    height: 46 * ghostTheme.s
                    placeholderText: "Username"
                    placeholderTextColor: ghostTheme.textSoft
                    color: ghostTheme.text
                    selectionColor: ghostTheme.accent
                    selectedTextColor: ghostTheme.background
                    font.family: ghostTheme.font
                    font.pixelSize: 15 * ghostTheme.s
                    leftPadding: 20 * ghostTheme.s
                    rightPadding: 20 * ghostTheme.s
                    Accessible.name: "Username"
                    onAccepted: password.forceActiveFocus()
                    background: Rectangle {
                        radius: height / 2
                        color: ghostTheme.field
                        border.width: usernameField.activeFocus ? Math.max(2, 2 * ghostTheme.s) : 1
                        border.color: usernameField.activeFocus ? ghostTheme.accentAlt : ghostTheme.border
                    }
                }

                Item { width: 1; height: 12 * ghostTheme.s; visible: root.manualUser }

                TextField {
                    id: password
                    objectName: "password"
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: 340 * ghostTheme.s
                    height: 46 * ghostTheme.s
                    echoMode: TextInput.Password
                    passwordCharacter: "•"
                    placeholderText: "Password"
                    placeholderTextColor: ghostTheme.textSoft
                    color: ghostTheme.text
                    selectionColor: ghostTheme.accent
                    selectedTextColor: ghostTheme.background
                    font.family: ghostTheme.font
                    font.pixelSize: 15 * ghostTheme.s
                    leftPadding: 20 * ghostTheme.s
                    rightPadding: 20 * ghostTheme.s
                    enabled: !root.busy
                    Accessible.name: "Password"
                    onAccepted: root.login()
                    onTextEdited: if (root.messageIsError) root.message = ""
                    transform: Translate { id: shakeOffset }
                    background: Rectangle {
                        radius: height / 2
                        color: ghostTheme.field
                        border.width: password.activeFocus ? Math.max(2, 2 * ghostTheme.s) : 1
                        border.color: password.activeFocus ? ghostTheme.accentAlt
                                    : (root.message !== "" && root.messageIsError ? ghostTheme.error : ghostTheme.border)
                    }
                }

                SequentialAnimation {
                    id: shake
                    loops: 2
                    NumberAnimation { target: shakeOffset; property: "x"; to: 6 * ghostTheme.s; duration: 45 }
                    NumberAnimation { target: shakeOffset; property: "x"; to: -6 * ghostTheme.s; duration: 90 }
                    NumberAnimation { target: shakeOffset; property: "x"; to: 0; duration: 45 }
                }

                // Fixed-height status line so messages do not shift the layout.
                Item {
                    width: 340 * ghostTheme.s
                    height: 34 * ghostTheme.s
                    anchors.horizontalCenter: parent.horizontalCenter

                    Text {
                        objectName: "message"
                        anchors.centerIn: parent
                        width: parent.width
                        horizontalAlignment: Text.AlignHCenter
                        elide: Text.ElideRight
                        text: root.message !== "" ? root.message
                            : (keyboard.capsLock ? "Caps Lock is on" : "")
                        color: root.message !== "" && root.messageIsError ? ghostTheme.error : ghostTheme.textSoft
                        font.family: ghostTheme.font
                        font.pixelSize: 13 * ghostTheme.s
                        Accessible.role: Accessible.AlertMessage
                        Accessible.name: text
                    }
                }

                PillButton {
                    id: loginButton
                    objectName: "loginButton"
                    theme: ghostTheme
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: 200 * ghostTheme.s
                    text: root.busy ? "Signing in…" : "Sign in"
                    enabled: !root.busy
                    onActivated: root.login()
                }
            }

            Row {
                anchors.horizontalCenter: parent.horizontalCenter
                anchors.bottom: parent.bottom
                anchors.bottomMargin: 22 * ghostTheme.s
                spacing: 10 * ghostTheme.s

                Text {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "Session"
                    color: ghostTheme.textSoft
                    font.family: ghostTheme.font
                    font.pixelSize: 13 * ghostTheme.s
                }

                ComboBox {
                    id: session
                    objectName: "session"
                    width: 210 * ghostTheme.s
                    height: 34 * ghostTheme.s
                    model: sessionModel
                    textRole: "name"
                    currentIndex: sessionModel.lastIndex
                    font.family: ghostTheme.font
                    font.pixelSize: 13 * ghostTheme.s
                    Accessible.name: "Session"

                    contentItem: Text {
                        leftPadding: 16 * ghostTheme.s
                        rightPadding: 34 * ghostTheme.s
                        verticalAlignment: Text.AlignVCenter
                        text: session.displayText
                        color: ghostTheme.text
                        font: session.font
                        elide: Text.ElideRight
                    }

                    indicator: Shape {
                        x: session.width - 14 * ghostTheme.s - 24 * ghostTheme.s * 0.7
                        y: (session.height - 24 * ghostTheme.s * 0.7) / 2
                        width: 24
                        height: 24
                        transformOrigin: Item.TopLeft
                        scale: ghostTheme.s * 0.7
                        preferredRendererType: Shape.CurveRenderer

                        ShapePath {
                            strokeColor: session.activeFocus || session.hovered ? ghostTheme.accentAlt : ghostTheme.accent
                            strokeWidth: 2.5
                            fillColor: "transparent"
                            capStyle: ShapePath.RoundCap
                            joinStyle: ShapePath.RoundJoin
                            PathSvg { path: "M6 9l6 6l6-6" }
                        }
                    }

                    background: Rectangle {
                        radius: height / 2
                        color: session.hovered ? Qt.rgba(1, 1, 1, 0.08) : ghostTheme.field
                        border.width: session.activeFocus ? Math.max(2, 2 * ghostTheme.s) : 1
                        border.color: session.activeFocus ? ghostTheme.accentAlt : ghostTheme.border
                        Behavior on color { ColorAnimation { duration: 120 } }
                    }

                    delegate: ItemDelegate {
                        id: sessionItem
                        required property int index
                        required property string name
                        width: session.width
                        height: 34 * ghostTheme.s
                        highlighted: session.highlightedIndex === index
                        contentItem: Text {
                            leftPadding: 8 * ghostTheme.s
                            verticalAlignment: Text.AlignVCenter
                            text: sessionItem.name
                            color: sessionItem.highlighted ? ghostTheme.accentAlt : ghostTheme.text
                            font: session.font
                            elide: Text.ElideRight
                        }
                        background: Rectangle {
                            radius: 10 * ghostTheme.s
                            color: sessionItem.highlighted
                                ? Qt.rgba(ghostTheme.accent.r, ghostTheme.accent.g, ghostTheme.accent.b, 0.16) : "transparent"
                        }
                    }

                    popup: Popup {
                        y: session.height + 6 * ghostTheme.s
                        width: session.width
                        padding: 4 * ghostTheme.s
                        implicitHeight: contentItem.implicitHeight + 2 * padding
                        contentItem: ListView {
                            clip: true
                            implicitHeight: contentHeight
                            model: session.popup.visible ? session.delegateModel : null
                            currentIndex: session.highlightedIndex
                        }
                        background: Rectangle {
                            radius: 14 * ghostTheme.s
                            color: ghostTheme.fieldSolid
                            border.width: 1
                            border.color: ghostTheme.border
                        }
                    }
                }
            }
        }

        Column {
            id: powerColumn
            width: sidebar.width
            anchors.verticalCenter: parent.verticalCenter
            spacing: 16 * ghostTheme.s
            enabled: root.pendingAction === ""
            opacity: enabled ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: 120 } }

            IconButton {
                id: suspendButton
                objectName: "suspendButton"
                theme: ghostTheme
                anchors.horizontalCenter: parent.horizontalCenter
                visible: sddm.canSuspend
                label: "Suspend"
                iconPath: "M20 14.5A8.5 8.5 0 1 1 9.5 4A7 7 0 0 0 20 14.5Z"
                onActivated: sddm.suspend()
            }

            IconButton {
                id: rebootButton
                objectName: "rebootButton"
                theme: ghostTheme
                anchors.horizontalCenter: parent.horizontalCenter
                visible: sddm.canReboot
                label: "Restart"
                iconPath: "M20 12A8 8 0 1 1 17.66 6.34M20 4V9H15"
                onActivated: root.requestPower("reboot")
            }

            IconButton {
                id: powerOffButton
                objectName: "powerOffButton"
                theme: ghostTheme
                anchors.horizontalCenter: parent.horizontalCenter
                visible: sddm.canPowerOff
                label: "Shut down"
                iconPath: "M12 3V11M6.34 6.34A8 8 0 1 0 17.66 6.34"
                onActivated: root.requestPower("poweroff")
            }
        }

        // Confirmation for shutdown and restart.
        Rectangle {
            id: confirm
            objectName: "confirm"
            anchors.fill: parent
            radius: panel.radius
            visible: root.pendingAction !== ""
            // The panel's own fill shows through; the controls underneath fade out.
            color: "transparent"

            Keys.onEscapePressed: root.closeConfirm()

            MouseArea {
                anchors.fill: parent
                // Swallow clicks so controls underneath cannot be used.
            }

            Column {
                anchors.centerIn: parent
                spacing: 26 * ghostTheme.s

                Text {
                    anchors.horizontalCenter: parent.horizontalCenter
                    text: root.pendingAction === "reboot" ? "Restart this computer?" : "Shut down this computer?"
                    color: ghostTheme.text
                    font.family: ghostTheme.font
                    font.pixelSize: 21 * ghostTheme.s
                    font.weight: Font.Medium
                }

                Row {
                    anchors.horizontalCenter: parent.horizontalCenter
                    spacing: 14 * ghostTheme.s

                    PillButton {
                        id: cancelButton
                        objectName: "cancelButton"
                        theme: ghostTheme
                        primary: false
                        text: "Cancel"
                        KeyNavigation.right: confirmButton
                        KeyNavigation.tab: confirmButton
                        Keys.onEscapePressed: root.closeConfirm()
                        onActivated: root.closeConfirm()
                    }

                    PillButton {
                        id: confirmButton
                        objectName: "confirmButton"
                        theme: ghostTheme
                        text: root.pendingAction === "reboot" ? "Restart" : "Shut down"
                        KeyNavigation.left: cancelButton
                        KeyNavigation.tab: cancelButton
                        Keys.onEscapePressed: root.closeConfirm()
                        onActivated: root.confirmPower()
                    }
                }
            }
        }
    }
}
