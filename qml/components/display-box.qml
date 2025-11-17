import QtQuick 2.15
import QtQuick.Controls 2.15

Item {
    id: root
    property bool showContent: true
    default property alias content: childrenRect.children
    visible: showContent

    Rectangle {
        id: childrenRect
        anchors.fill: parent
        color: "transparent"
    }
}
