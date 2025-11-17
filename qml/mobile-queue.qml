import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import dataflow 1.0  // For SlintMediaFile

Item {
    id: root
    property var queue: list  // List of SlintMediaFile

    // Title
    Label {
        text: qsTr("Mobile queue")
        font.pixelSize: 18
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: 10
    }

    // List of media files
    ListView {
        id: listView
        anchors.top: title.bottom
        anchors.topMargin: 10
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        clip: true

        model: queue

        delegate: ItemDelegate {
            width: ListView.view.width
            text: modelData.name
            onClicked: {
                // Handle click event if needed
                console.log("Clicked on media file:", modelData.name);
            }
        }
    }
}
