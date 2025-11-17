import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import dataflow 1.0  // For SlintMediaFile, MediaActions, SlintState
import style 1.0     // For Palette

Item {
    id: root
    property var queue: list  // List of SlintMediaFile

    // Title
    Label {
        text: qsTr("Queue")
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
        height: parent.height - title.height - 10
        clip: true
        model: queue

        delegate: Item {
            width: ListView.view.width
            height: 30

            // TouchArea for click handling
            MouseArea {
                anchors.fill: parent
                onClicked: {
                    MediaActions.mediaStart(modelData.id);
                }
            }

            // Background rectangle
            Rectangle {
                anchors.fill: parent
                color: Palette.secondaryBackground

                // Horizontal layout for item info
                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 5
                    spacing: 10

                    // Item ID
                    Label {
                        text: modelData.id
                    }

                    // Item name
                    Label {
                        text: modelData.name
                        Layout.fillWidth: true
                    }
                }
            }
        }
    }
}
