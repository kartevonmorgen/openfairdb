-- Fix invalid millisecond time stamps that have been accepted before v0.8.0
update events set start=start/1000 where start>1000000000000;
update events set end=end/1000 where end not null and end>1000000000000;
