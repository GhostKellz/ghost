import QtQuick
import QtQuick.Shapes

// Round, single-color icon button. The icon is an SVG path on a 24x24 grid so
// it stays sharp at any scale and recolors without per-state image assets.
Item {
    id: control

    required property QtObject theme
    property string iconPath
    property string label
    signal activated()

    readonly property bool lit: mouse.containsMouse || activeFocus

    width: 52 * theme.s
    height: width
    activeFocusOnTab: true
    opacity: enabled ? 1 : 0.4

    Accessible.role: Accessible.Button
    Accessible.name: label
    Accessible.onPressAction: activated()

    Keys.onReturnPressed: activated()
    Keys.onEnterPressed: activated()
    Keys.onSpacePressed: activated()

    Rectangle {
        anchors.fill: parent
        radius: 14 * control.theme.s
        color: control.lit ? Qt.rgba(control.theme.accent.r, control.theme.accent.g, control.theme.accent.b, 0.14) : "transparent"
        border.width: control.activeFocus ? Math.max(2, 2 * control.theme.s) : 0
        border.color: control.theme.accentAlt
        Behavior on color { ColorAnimation { duration: 120 } }
    }

    Shape {
        width: 24
        height: 24
        anchors.centerIn: parent
        scale: control.theme.s
        preferredRendererType: Shape.CurveRenderer

        ShapePath {
            strokeColor: control.lit ? control.theme.accentAlt : control.theme.accent
            strokeWidth: 2
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            PathSvg { path: control.iconPath }
        }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: control.activated()
    }

    // Label to the right of the button, shown on hover or keyboard focus.
    Rectangle {
        visible: control.lit && control.label !== ""
        x: parent.width + 10 * control.theme.s
        anchors.verticalCenter: parent.verticalCenter
        width: tipText.implicitWidth + 20 * control.theme.s
        height: tipText.implicitHeight + 10 * control.theme.s
        radius: height / 2
        color: control.theme.surface
        border.width: 1
        border.color: control.theme.border
        z: 10

        Text {
            id: tipText
            anchors.centerIn: parent
            text: control.label
            color: control.theme.text
            font.family: control.theme.font
            font.pixelSize: 13 * control.theme.s
        }
    }
}
