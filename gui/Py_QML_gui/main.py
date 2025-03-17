# This Python file uses the following encoding: utf-8
from __future__ import annotations
import sys
import json
from pathlib import Path

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

class Netstuff(QObject):
    def __init__(self):
        super().__init__()
        #need to create instances here for acting upon later
        self.nam = QNetworkAccessManager(self)
        self.nam.finished.connect(self.processReply) #means connect finished signal of net manager calls the processReply
        #get handles and assign the windows to the plots
        self.win1 = pg.GraphicsLayoutWidget(title="Client 1 Voltage (mV) vs Time (ms)")
        self.win2 = pg.GraphicsLayoutWidget(title="Client 2 Voltage (mV) vs Time (ms)")
        self.win3 = pg.GraphicsLayoutWidget(title="Client 3 Voltage (mV) vs Time (ms)")
        self.win4 = pg.GraphicsLayoutWidget(title="Client 4 Voltage (mV) vs Time (ms)")
        self.plot1 = self.win1.addPlot(title="Client 1 Voltage vs Time")
        self.plot2 = self.win2.addPlot(title="Client 2 Voltage vs Time")
        self.plot3 = self.win3.addPlot(title="Client 3 Voltage vs Time")
        self.plot4 = self.win4.addPlot(title="Client 4 Voltage vs Time")
        #Show windows
        self.win1.show()
        self.win2.show()
        self.win3.show()
        self.win4.show()


    @Slot()
    def sendRequest(self):
        url = QUrl("http://192.168.2.1:8080")
        request = QNetworkRequest(url)
        self.reply = self.nam.get(request) #calls get on the selfs request variable (defined above)

    @Slot()
    def processReply(self, reply: QNetworkReply): #two arguments, needs to recieve reply
        er = reply.error()
        if er == QNetworkReply.NoError:
            answerAsText = bytes(reply.readAll()).decode("utf-8") #decode to text for turning into graph

            readings = json.loads(answerAsText)
            print(readings)

            #here i want to put the values i get from readings into the plots declared above
            data1 = readings["client1"].get("readings", [])
            times1 = [item["sensor_time"] for item in data1]
            values1 = [item["sensor_value"] for item in data1]
            self.plot1.clear()  # clear previous data
            self.plot1.plot(times1, values1, pen='r', symbol='o')

            data2 = readings["client2"].get("readings", [])
            times2 = [item["sensor_time"] for item in data2]
            values2 = [item["sensor_value"] for item in data2]
            self.plot2.clear()  # clear previous data
            self.plot2.plot(times2, values2, pen='r', symbol='o')

            data3 = readings["client3"].get("readings", [])
            times3 = [item["sensor_time"] for item in data3]
            values3 = [item["sensor_value"] for item in data3]
            self.plot3.clear()  # clear previous data
            self.plot3.plot(times3, values3, pen='r', symbol='o')

            data4 = readings["client4"].get("readings", [])
            times4 = [item["sensor_time"] for item in data4]
            values4 = [item["sensor_value"] for item in data4]
            self.plot4.clear()  # clear previous data
            self.plot4.plot(times4, values4, pen='r', symbol='o')

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
    timer.start(1000) #timer repeats every 500ms


    #engine.rootContext().setContextProperty("backend", backend) #expose backend to qml


    #qml_file = Path(__file__).resolve().parent / "main.qml" # create engine instance and load file to engine object
    #engine.load(qml_file)
    #if not engine.rootObjects(): #checks if load is succesful
    #    sys.exit(-1)
    sys.exit(app.exec())
