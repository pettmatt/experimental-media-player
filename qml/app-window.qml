import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

import "components"  // For DisplayBox, Queue, SideBar, MainList, MediaControllers, MobileQueue

ApplicationWindow {
    id: root
    visible: true
    width: 800
    height: 600
    title: qsTr("Media App")

    // Define properties for margins and padding
    property real glMargin: 5
    property real glPadding: 5

    // Define the SlintState, SettingActions, and MediaActions
    // These should be registered as QObjects in C++/Rust
    property var SlintState: QState {}
    property var SettingActions: QSettingActions {}
    property var MediaActions: QMediaActions {}

    // Main horizontal layout
    Item {
        id: mainView
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: root.height - (root.width < 700 ? 200 : 150)

        RowLayout {
            anchors.fill: parent
            spacing: glMargin

            // MainList (always visible)
            MainList {
                Layout.minimumWidth: mainView.width * 0.6
                Layout.fillWidth: true
                list: SlintState.index
            }

            // Queue (visible only if root.width > 700)
            DisplayBox {
                visible: root.width > 700
                Queue {
                    queue: SlintState.queue
                }
            }
        }
    }

    // Vertical layout at the bottom
    Item {
        id: verticalView
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: root.width < 700 ? 200 : 150

        ColumnLayout {
            anchors.fill: parent
            spacing: glMargin

            // Test button
            Button {
                text: qsTr("Test")
                onClicked: SettingActions.newLocalSource()
            }

            // SideBar (visible only if root.width < 500)
            SideBar {
                visible: root.width < 500
                list: SlintState.playlist
            }

            // MobileQueue (visible only if root.width < 700)
            MobileQueue {
                visible: root.width < 700
                queue: SlintState.queue
            }

            // MediaControllers (always visible)
            MediaControllers {
                Layout.fillWidth: true
                playing: SlintState.timeline
            }
        }
    }
}
