-module(init).
-export([start/0]).
-import(erlang, [demonitor/2, display/1, make_ref/0]).

start() ->
  Reference = make_ref(),
  Options = [flush, info],
  display(demonitor(Reference, Options)).
