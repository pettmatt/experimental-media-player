import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Dialogs 1.3
import QtQuick.Layouts 1.15

Item {
    id: root
    property var sourcePaths: []  // List of source paths

    // Signal to request a new source path
    signal requestNewSourcePath(string sourcePath)

    ColumnLayout {
        anchors.fill: parent
        spacing: 10

        // Directory input row
        RowLayout {
            spacing: 10

            // Label
            Label {
                text: qsTr("Source directories")
                Layout.alignment: Qt.AlignVCenter
            }

            // Text input for directory path
            TextField {
                id: dirInput
                placeholderText: qsTr("Directory path")
                Layout.fillWidth: true
            }

            // Button to open file dialog
            Button {
                text: qsTr("Find dir")
                onClicked: {
                    fileDialog.open();
                }
            }

            // Button to add directory
            Button {
                text: qsTr("Add")
                onClicked: {
                    if (dirInput.text !== "") {
                        root.requestNewSourcePath(dirInput.text);
                        dirInput.text = "";
                    }
                }
            }
        }

        // List of source paths
        RowLayout {
            spacing: 10
            wrapMode: RowLayout.Wrap
            Layout.fillWidth: true

            // Display each path
            Repeater {
                model: root.sourcePaths

                Rectangle {
                    Layout.preferredWidth: 200
                    Layout.preferredHeight: 30
                    border.color: "lightgray"
                    radius: 3

                    Label {
                        anchors.fill: parent
                        anchors.margins: 5
                        text: modelData
                        wrapMode: Text.Wrap
                    }
                }
            }
        }
    }

    // File dialog for selecting directories
    FileDialog {
        id: fileDialog
        title: qsTr("Select Directory")
        folder: shortcuts.home
        selectFolder: true
        onAccepted: {
            dirInput.text = fileDialog.fileUrl.toLocalFile();
        }
    }
}
