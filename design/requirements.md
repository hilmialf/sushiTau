# Assignment

*Business Case*

Create a restaurant application which accepts menu items from various serving staff in the restaurant.  This
application must then store the item along with a cooking time for the item to be completed.  The application
must be able to give a quick snapshot of any or all items on its list at any time.  It must also be able to remove
specific orders from the list of orders on demand.

## System Actors

### The application
Running on a “server” and accepting calls from devices carried by restaurant staff to process guest’s
menu orders.  This is where the bulk of time should be spent.

### The client
Multiple "tablets" carried by restaurant staff to take orders.  These will send requests to the “server”
to add, remove, and query menu items for each table.  Please make this as simple as possible.

## Requirements
* The client (the restaurant staff “devices” making the requests) MUST be able to: add one or more items with a
  table number, remove an item for a table, and query the items still remaining for a table.
* The application MUST, upon creation request, store the item, the table number, and how long the item will take to cook.
* The application MUST, upon deletion request, remove a specified item for a specified table number.
* The application MUST, upon query request, show all items for a specified table number.
* The application MUST, upon query request, show a specified item for a specified table number.
* The application MUST accept at least 10 simultaneous incoming add/remove/query requests.
* The client MAY limit the number of specific tables in its requests to a finite set (at least 100).
* The application MAY assign a length of time for the item to prepare as a random time between 5-15 minutes.
* The application MAY keep the length of time for the item to prepare static (in other words, the time does not have
  to be counted down in real time, only upon item creation and then removed with the item upon item deletion).

### Allowed Assumptions

You may have your application assume the following to simplify the solution, if desired:

* The time to prepare does not have to be kept up-to-date.  It can also just be generated as some random amount
  of time between 5 and 15 minutes and kept static from then on.
* The table and items can be identified in any chosen manner, but it has to be consistent. So if a request comes in
  for table "4", for example, any other requests for table "4" must refer to the same table.
* “Clients” can be simulated as simple threads in a main() function calling the main server application with a
  variety of requests.  There should be more than one, preferably around 5-10 running at any one time.
* The API is up to the developer.  HTTP REST is acceptable, but direct API calls are also acceptable if they mimic an
  HTTP REST-like API (e.g. api_call1(string id, string resource), etc.).

