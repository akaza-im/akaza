from comb.ui import CombIBusEngine


def test_extend_clause_right():
    ui = CombIBusEngine()
    ui.preedit_string = "tanosiijikan"  # 楽し/い/時間 になるはず
    ui.update_candidates()

    # AS-IS:
    #   タノシ/イ/ジカン
    #   ↑ focus
    # TO-BE:
    #   タノシ/イ/ジカン
    #         ↑ focus
    ui.cursor_right()

    # 「イジ」に伸びるはず
    # タノシ/イジ/カン
    # 0 1 2 3 4 5 6
    # slice(0,3)
    # slice(3,5)
    # slice(5,7)
    ui.extend_clause_right()

    print(ui.build_string())
    print(ui.force_selected_clause)
    got = '/'.join(["タノシイジカン"[s] for s in ui.force_selected_clause])
    assert got == 'タノシ/イジ/カン'

    # 2文節目が イジ になっている
    assert '維持' in [node.word for node in ui.clauses[1]]


def test_extend_clause_left():
    ui = CombIBusEngine()
    ui.preedit_string = "tanosiijikan"  # 楽し/い/時間 になるはず
    ui.update_candidates()

    # AS-IS:
    #   タノシ/イ/ジカン
    #   ↑ focus
    # TO-BE:
    #   タノシ/イ/ジカン
    #         ↑ focus
    ui.cursor_right()

    print('/'.join([clause[0].yomi for clause in ui.clauses]))

    # タノ/シイ/ジカン
    # 0 1 2 3 4 5 6
    # slice(0,2)
    # slice(2,4)
    # slice(4,7)
    ui.extend_clause_left()

    print(ui.build_string())
    print(ui.force_selected_clause)
    got = '/'.join(["タノシイジカン"[s] for s in ui.force_selected_clause])
    assert got == 'タノ/シイ/ジカン'

    print('/'.join([clause[0].yomi for clause in ui.clauses]))

    # 2文節目が しい になっている
    assert 'たの/しい/じかん' == '/'.join([clause[0].yomi for clause in ui.clauses])
