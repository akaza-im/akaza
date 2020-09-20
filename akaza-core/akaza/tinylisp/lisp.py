import datetime


# 簡易 LISP のインタープリタ for akaza。
# No variables.
# ref. http://norvig.com/lispy.html

class Symbol:
    def __init__(self, s: str):
        self.s = s

    def __str__(self):
        return self.s

    def __repr__(self):
        return '#' + self.s

    def __eq__(self, other):
        return self.s == other.s

    def __hash__(self):
        return self.s.__hash__()


class Env:
    def __init__(self, outer=None):
        self.outer = outer
        self.d = {}

    def get(self, var):
        if var in self.d:
            return self.d[var]
        if self.outer:
            return self.outer.get(var)
        raise RuntimeError(f"Cannot get variable(akaza-tinylisp): {var}")

    def __setitem__(self, key, value):
        self.d[key] = value


class Evaluator:
    def __init__(self):
        self.global_env = Env()
        self.global_env[Symbol('+')] = lambda a, b: a + b
        self.global_env[Symbol('current-datetime')] = lambda: datetime.datetime.now()
        self.global_env[Symbol('strftime')] = lambda dt, fmt: dt.strftime(fmt)

    def eval(self, x, env=None):
        if env is None:
            env = self.global_env

        if isinstance(x, Symbol):  # get variable
            return env.get(x)
        elif not isinstance(x, list):  # return list itself
            return x
        else:  # (proc exp*)
            exps = [self.eval(exp, env) for exp in x]
            proc = exps.pop(0)
            return proc(*exps)

    def run(self, sexp):
        return self.eval(parse(sexp))


def parse(s):
    return _read_from(_tokenize(s))


def _tokenize(s):
    return s.replace('(', ' ( ').replace(')', ' ) ').split()


def _read_from(tokens):
    if len(tokens) == 0:
        raise SyntaxError('unexpected EOF while reading')
    token = tokens.pop(0)
    if '(' == token:
        values = []
        while tokens[0] != ')':
            values.append(_read_from(tokens))
        tokens.pop(0)  # pop off ')'
        return values
    elif ')' == token:
        raise SyntaxError('unexpected )')
    else:
        return _atom(token)


def _atom(token):
    try:
        return int(token)
    except ValueError:
        try:
            return float(token)
        except ValueError:
            if token.startswith('"'):
                return token[1:-1]
            else:
                return Symbol(token)
