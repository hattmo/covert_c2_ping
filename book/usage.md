# Usage

## Listener Setup
To create an External C2 Beacon listener select Cobalt Strike -> Listeners on the main
 menu and press the Add button at the bottom of the Listeners tab display. Select the
 external c2 payload.

Note the port number and host of the cobalt strike team server listener you created

## Start Server

Start the covert c2 server either on the same box or another box, but note that the c2
server will need to receive icmp messages and bind to port 8080

```bash
sudo ./covert_c2_ping --ts [hostname]:[port]
```

additionally you may adjust what port the server will listen for icmp messages on with
 the `-i` flag.

## Generate  Artifact

To create an artifact navigate to the host that covert c2 is running and port `8080`
with your browser.

On the page presented enter the callback `Host` of this c2 server NOT 
the cobalt strike team server (though they may be the same).  This value may be different 
if you expect the binary to reach back to this server via a proxy or some other routing
 magic.

`Pipe` is the name of the named pipe that the artifact will use with the beacon it deploys
 at run time. the artifact will read from this pipe and forward the messages via ping to
 the covert c2 server which is then forwarded to the cobalt strike team server.

`Sleep` is the initial call back interval.  By default the artifact will send a ping every 
two seconds which is the standard interval for the built in ping command.

When completed click the create button to down load the artifact.

## Executing Artifact

Once the artifact is generated a session is started with the cobalt strike server expecting
a response from this particular artifact.  an entry in the table will be added showing 
the status of this artifact  THE INDIVIDUAL ARTIFACT CAN ONLY BE RUN ONCE and is 
associated with a unique session with the CS Team Server. If you need to run another instance
 regenerate a new artifact with the covert c2 server.  If a artifact calls back correctly,
 It should appear in the covert c2 server list and the cobaltstrike team server list.

 To adjust the interval that the artifact sends icmp messages click the 🛠️ icon.  To remove
 an running artifact click the 🗑️ icon.