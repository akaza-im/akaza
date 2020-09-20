import datetime
from akaza import tinylisp


def test_1():
    ast = tinylisp.parse('(+ 1 2)')
    assert ast == [tinylisp.Symbol('+'), 1, 2]
    evaluator = tinylisp.Evaluator()
    got = evaluator.eval(ast)
    assert got == 3


def test_date():
    evaluator = tinylisp.Evaluator()
    assert evaluator.eval(tinylisp.parse('(strftime (current-datetime) "%Y-%m-%d")')) == \
           datetime.datetime.now().strftime('%Y-%m-%d')
