"""The broker's guarantees, without Docker: the only command it builds, the
routes it answers, and timeout clamping."""

import json
import threading
import unittest
import urllib.error
import urllib.request
from http.server import ThreadingHTTPServer

import broker

CFG = {"host_root": "/srv/robots", "image": "drivesim-import:latest", "runtime": "runsc",
       "cpus": "2", "memory": "4g", "bind": "127.0.0.1:0", "parallel": "1"}
RID = "0123456789abcdef0123456789abcdef"


class Argv(unittest.TestCase):
    def test_sandbox_flags_and_mounts(self):
        a = broker.import_argv(RID, 600, CFG)
        for flag in ("--network=none", "--read-only", "--cap-drop=ALL", "--rm"):
            self.assertIn(flag, a)
        self.assertEqual(a[a.index("--runtime") + 1], "runsc")
        mounts = [a[i + 1] for i, x in enumerate(a) if x == "-v"]
        self.assertEqual(mounts, [f"/srv/robots/{RID}/robot.step:/work/robot.step:ro",
                                  f"/srv/robots/{RID}/bundle:/work/bundle:rw"])
        self.assertIn("timeout -s KILL 600 python3 /opt/import/step_import.py", a[-1])
        self.assertTrue(a[-1].startswith("umask 0 && "))

    def test_ids_must_be_32_hex(self):
        with self.assertRaises(AssertionError):
            broker.import_argv("../etc", 600, CFG)

    def test_timeouts_are_clamped(self):
        self.assertEqual(broker.clamp_timeout(1), broker.TIMEOUT_MIN)
        self.assertEqual(broker.clamp_timeout(10**9), broker.TIMEOUT_MAX)
        self.assertEqual(broker.clamp_timeout("x"), broker.TIMEOUT_DEFAULT)


class Http(unittest.TestCase):
    def setUp(self):
        self.calls = []

        def fake(rid, t, cfg):
            self.calls.append((rid, t))
            return {"code": 0, "stdout": "ok", "stderr": ""}

        handler = broker.make_handler(CFG, threading.Semaphore(1), runner=fake)
        self.srv = ThreadingHTTPServer(("127.0.0.1", 0), handler)
        threading.Thread(target=self.srv.serve_forever, daemon=True).start()
        self.url = f"http://127.0.0.1:{self.srv.server_address[1]}"

    def tearDown(self):
        self.srv.shutdown()

    def post(self, path, body=b"{}"):
        req = urllib.request.Request(self.url + path, data=body, method="POST")
        try:
            with urllib.request.urlopen(req) as r:
                return r.status, json.loads(r.read())
        except urllib.error.HTTPError as e:
            with e:
                return e.code, json.loads(e.read())

    def test_run_reaches_the_runner_with_a_clamped_timeout(self):
        self.assertEqual(self.post(f"/imports/{RID}/run", b'{"timeout": 99999}')[0], 200)
        self.assertEqual(self.calls, [(RID, broker.TIMEOUT_MAX)])

    def test_anything_else_is_404(self):
        for path in ("/imports/../run", f"/imports/{RID}/exec", "/", f"/imports/{RID.upper()}/run"):
            self.assertEqual(self.post(path)[0], 404, path)
        self.assertEqual(self.calls, [])


if __name__ == "__main__":
    unittest.main()
