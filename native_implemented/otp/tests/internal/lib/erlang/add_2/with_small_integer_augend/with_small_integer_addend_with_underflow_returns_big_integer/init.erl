-module(init).
-export([start/0]).
-import(erlang, [display/1]).
-import(lumen, [is_big_integer/1, is_small_integer/1]).

start() ->
  Sum = augend() + addend(),
  display(is_big_integer(Sum)).

augend() ->
  SmallInteger = -99999999999999,
  display(is_small_integer(SmallInteger)),
  SmallInteger.

addend() ->
  SmallInteger = -99999999999999,
  display(is_small_integer(SmallInteger)),
  SmallInteger.
