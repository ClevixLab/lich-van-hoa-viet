# -*- coding: utf-8 -*-
"""In luoc do va vai dong cua mot tep SQLite (doc nhu DU LIEU, che do chi doc)."""
import sqlite3, sys
c = sqlite3.connect("file:%s?mode=ro" % sys.argv[1], uri=True)
print("user_version", c.execute("pragma user_version").fetchone())
for (s,) in c.execute("select sql from sqlite_master where sql is not null"):
    print(s)
for (t,) in c.execute("select name from sqlite_master where type='table'"):
    rows = c.execute("select * from %s" % t).fetchall()
    print("--", t, len(rows))
    for r in rows[:2]:
        print("  ", [str(x)[:160] for x in r])
