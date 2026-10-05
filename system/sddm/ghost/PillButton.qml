import QtQuick

Item {
    id: control

    required property QtObject theme
    property string text
    property bool primary: true
    signal activated()

    readonly property bool hovered: mouse.containsMouse

    implicitWidth: Math.max(140 * theme.s, label.implicitWidth + 48 * theme.s)
    implicitHeight: 44 * theme.s
    activeFocusOnTab: true
    opacity: enabled ? 1 : 0.55

    Accessible.role: Accessible.Button
    Accessible.name: text
    Accessible.onPressAction: activated()

    Keys.onReturnPressed: activated()
    Keys.onEnterPressed: activated()
    Keys.onSpacePressed: activated()

    // Focus ring sits outside the fill so it stays visible on the blue button.
    Rectangle {
        anchors.fill: parent
        anchors.margins: -4 * control.theme.s
        radius: height / 2
        color: "transparent"
        border.width: Math.max(2, 2 * control.theme.s)
        border.color: control.theme.accentAlt
        visible: control.activeFocus
    }

    Rectangle {
        anchors.fill: parent
        radius: height / 2
        color: control.primary
            ? (control.hovered ? Qt.lighter(control.theme.accent, 1.08) : control.theme.accent)
            : (control.hovered ? Qt.rgba(1, 1, 1, 0.10) : Qt.rgba(1, 1, 1, 0.06))
        border.width: control.primary ? 0 : 1
        border.color: control.theme.border
        Behavior on color { ColorAnimation { duration: 120 } }
    }

    Text {
        id: label
        anchors.centerIn: parent
        text: control.text
        // Dark text on the blue fill: white on #7aa2f7 fails contrast.
        color: control.primary ? control.theme.background : control.theme.text
        font.family: control.theme.font
        font.pixelSize: 15 * control.theme.s
        font.weight: Font.DemiBold
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: if (control.enabled) control.activated()
    }
}
