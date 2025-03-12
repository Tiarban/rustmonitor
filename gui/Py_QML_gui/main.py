# This Python file uses the following encoding: utf-8
import sys
from pathlib import Path
from __future__ import annotations


from PySide6.QtWidgets import QApplication  # Changed from QGuiApplication to QApplication
from PySide6.QtQml import QQmlApplicationEngine #access to qml

from PySide6.QtWidgets import (
    QWidget,
    QApplication,
    QMessageBox,
    QLineEdit,
    QProgressBar,
    QPushButton,
    QHBoxLayout,
    QVBoxLayout,
    QStyle,
    QFileDialog,
)
from PySide6.QtCore import QStandardPaths, QUrl, QFile, QSaveFile, QDir, QIODevice, Slot, QTimer
from PySide6.QtNetwork import QNetworkReply, QNetworkRequest, QNetworkAccessManager

import pyqtgraph as pg
from PySide6.QtCore import QObject, Slot

class Backend(QObject): #graph stuff in this class
    def __init__(self):
        super().__init__()
        # Keep a reference to the graph window to prevent garbage collection.
        self._graph_window = None


    @Slot()
    def showGraph(self):
        # Create a new window for the plot using pyqtgraph
        win = pg.GraphicsLayoutWidget(title="Temperature vs Time")
        plot = win.addPlot(title="Temperature vs Time")
        time = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        temperature = [30, 32, 34, 32, 33, 31, 29, 32, 35, 30]
        plot.plot(time, temperature, pen='r')
        win.show()
        self._graph_window = win  # store reference

class Netstuff(QObject):
    def __init__(self):
        super().__init__()
        #need to create instances here for acting upon later
        self.nam = QNetworkAccessManager(self)
        self.nam.finished.connect(self.processReply) #means connect finished signal of net manager calls the processReply


    @Slot()
    def sendRequest(self):
        url = QUrl("http://192.168.2.1:8080")
        request = QNetworkRequest(url)
        self.reply = self.nam.get(request) #calls get on the selfs request variable (defined above)

    @Slot()
    def processReply(self, reply: QNetworkReply): #two arguments, needs to recieve reply
        er = reply.error()
        if er == QtNetwork.QNetworkReply.NoError:
            answerAsText = bytes(reply.readAll()).decode("utf-8") #decode to text for turning into graph
            print(answerAsText)
        else:
            print("Error occured: ", er)
            print(reply.errorString())
        reply.deleteLater() #cleans up resources


if __name__ == "__main__":
    app = QApplication(sys.argv) #creates instance of qgui and passes system to it
    engine = QQmlApplicationEngine()

    netstuff = Netstuff()
    timer = QTimer() #initialse timer for repeated network requests
    timer.timeout.connect(netstuff.sendRequest) #pipes the timeout signal to send request (and therefore process reply)
    time.start(500) #timer repeats every 500ms

    backend = Backend() #make graph stuff
    engine.rootContext().setContextProperty("backend", backend) #expose backend to qml


    qml_file = Path(__file__).resolve().parent / "main.qml" # create engine instance and load file to engine object
    engine.load(qml_file)
    if not engine.rootObjects(): #checks if load is succesful
        sys.exit(-1)
    sys.exit(app.exec())
