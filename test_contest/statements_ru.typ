#let lang = "ru"

#let tr = (
  en: (
    page: "1 of 1",
    tl: "Time limit",
    ml: "Memory limit",
    input: "Input format",
    output: "Output format",
    interactive_protocol: "Interactive protocol",
    first_run: "The first run",
    second_run: "The second run",
    samples: "Samples",
    run: "Run",
    stdin: "stdin",
    stdout: "stdout",
    interactor: "Interactor",
    solution: "Solution",
    comments: "Comments",
    scoring: "Scoring",
    subgroup: "Subgroup",
    limits: "Limits",
    score: "Score",
    depends_on: "Required subgroups",
  ),
  ru: (
    page: "Страница 1 из 1",
    tl: "Ограничение по времени",
    ml: "Ограничение по памяти",
    input: "Входные данные",
    output: "Выходные данные",
    interactive_protocol: "Протокол взаимодействия",
    first_run: "Первый запуск",
    second_run: "Второй запуск",
    samples: "Примеры",
    run: "Запуск",
    stdin: "Ввод",
    stdout: "Вывод",
    interactor: "Интерактор",
    solution: "Решение",
    comments: "Примечание",
    scoring: "Система оценки",
    subgroup: "Подгруппа",
    limits: "Ограничения",
    score: "Баллы",
    depends_on: "Требуемые подгруппы",
  )
)

#let t(key) = tr.at(lang).at(key)

#let template(title, subtitle: none, body) = {
  set page(
    paper: "a5",
    margin: (x: 20pt, top: 100pt, bottom: 50pt),
    header: align(center)[
      #line(length: 100%)
      == #title
      #subtitle
      #line(length: 100%)
    ],
    footer: context align(center)[
      #line(length: 100%)
      #counter(page).display(t("page"), both: true)
    ],
  )

  set par(justify: true, linebreaks: "optimized")

  body
}

// Сэмплы лучше передавать строками: raw сохраняет пробелы и переводы строк,
// поэтому пример одинаково выглядит в PDF и нормально копируется. Content
// оставлен для совместимости со старыми условиями.
#let sample-code(body) = {
  set text(font: "DejaVu Sans Mono", size: 8pt)
  set par(justify: false, linebreaks: "simple")
  if type(body) == str {
    raw(body, block: false, lang: "text")
  } else {
    body
  }
}

#let problem(title,
             tl,
             ml,
             statement,
             input: none,
             output: none,
             interactive_protocol: none,
             first_run: none,
             second_run: none,
             samples,
             comments: none,
             subgroups_scoring: none,
             per_test_scoring: none,
             sample_headers: none) = {
  let double-run = first_run != none and second_run != none
  let interactive = interactive_protocol != none
  let headers = if sample_headers != none {
    sample_headers
  } else if interactive {
    (t("interactor"), t("solution"))
  } else {
    (t("stdin"), t("stdout"))
  }

  [
    = #title
    #table(
      columns: (auto, auto),
      align: center,
      [
        === #t("tl")
      ],
      [
        #tl
      ],

      [
        === #t("ml")
      ],
      [
        #ml
      ],
    )
    #statement
    #if input != none {
      [
        == #t("input")
        #input
      ]
    }
    #if output != none {
      [
        == #t("output")
        #output
      ]
    }
    #if interactive_protocol != none {
      [
        == #t("interactive_protocol")
        #interactive_protocol
      ]
    }
    #if first_run != none {
      [
        == #t("first_run")
        #first_run
      ]
    }
    #if second_run != none {
      [
        == #t("second_run")
        #second_run
      ]
    }
    #if samples.len() > 0 {
      [
        == #t("samples")
        #{
          let columns = if double-run {
            (auto, auto, 1fr, 1fr)
          } else {
            (auto, 1fr, 1fr)
          }
          let align = if double-run {
            (center, left, left, left)
          } else {
            (center, left, left)
          }
          let header = if double-run {
            ("#", t("run"), headers.at(0), headers.at(1))
          } else {
            ("#", headers.at(0), headers.at(1))
          }
          let cells = range(0, samples.len(), step: 2).map(i => {
            let pair = calc.quo(i, 2)
            let number = if double-run { calc.quo(i, 4) + 1 } else { pair + 1 }
            let output = if i + 1 < samples.len() { samples.at(i + 1) } else { [] }
            if double-run {
              let run = if calc.rem(pair, 2) == 0 {
                t("first_run")
              } else {
                t("second_run")
              }
              (
                [#str(number)],
                [#run],
                sample-code(samples.at(i)),
                sample-code(output),
              )
            } else {
              ([#str(number)], sample-code(samples.at(i)), sample-code(output))
            }
          }).flatten()
          table(
            columns: columns,
            inset: (x: 5pt, y: 4pt),
            align: align,
            table.header(..header),
            ..cells,
          )
        }
      ]
    }
    #if comments != none {
      [
        == #t("comments")
        #comments
      ]
    }
    #if subgroups_scoring != none {
      [
        == #t("scoring")
        #table(
          columns: (auto, 1fr, auto, 1fr),
          inset: 4pt,
          align: (center, left, center, left),
          table.header(t("subgroup"), t("limits"), t("score"), t("depends_on")),
          ..subgroups_scoring.flatten(),
        )
      ]
    }
    #if per_test_scoring != none {
      [
        == #t("scoring")
        #per_test_scoring
      ]
    }
  ]
}

#template([Тестовый контест], subtitle: [давным-давно],
  [
    #problem(
      "A. Сумма чисел I",
      "1 секунда",
      "256 мегабайт",
      [
        Даны целые числа $n$ и $m$. Вычислите их сумму.
      ],
      input:
      [
        В единственной строке записаны целые числа $n$ и $m$ $(1 <= n, m <= 1000)$.
      ],
      output:
      [
        Выведите одно целое число — сумму $n + m$.
      ],
      (
        "3 4",
        "7",
      ),
      subgroups_scoring:
      (
        "0", "Тесты из условия", "0", "—",
        "1", $(1 <= n, m <= 1000)$, "100", "0",
      ),
    )

    #pagebreak()

    #problem(
      "B. Сумма чисел II",
      "1 секунда",
      "256 мегабайт",
      [
        Даны целые числа $n$ и $m$. Вычислите их сумму.
      ],
      input:
      [
        В единственной строке записаны целые числа $n$ и $m$ $(1 <= n, m <= 1000)$.
      ],
      output:
      [
        Выведите одно целое число — сумму $n + m$.
      ],
      (
        "3 4",
        "7",
      ),
      subgroups_scoring:
      (
        "0", "Тесты из условия", "0", "—",
        "1", $(1 <= n, m <= 500)$, "50", "0",
        "2", $(1 <= n, m <= 1000)$, "50", "—",
      ),
    )

    #pagebreak()

    #problem(
      "C. Сумма чисел III",
      "1 секунда",
      "256 мегабайт",
      [
        Даны целые числа $n$ и $m$. Вычислите их сумму.
      ],
      input:
      [
        В единственной строке записаны целые числа $n$ и $m$ $(1 <= n, m <= 1000)$.
      ],
      output:
      [
        Выведите одно целое число — сумму $n + m$.
      ],
      (
        "3 4",
        "7",
      ),
      subgroups_scoring:
      (
        "0", "Тесты из условия", "0", "—",
        "1", $(1 <= n, m <= 1000)$, "50", "179",
      ),
    )

    #pagebreak()

    #problem(
      "D. Сумма чисел IV",
      "1 секунда",
      "256 мегабайт",
      [
        *Это задача с объединением подгрупп.*\
        Даны целые числа $n$ и $m$. Вычислите их сумму.
      ],
      input:
      [
        В единственной строке записаны целые числа $n$ и $m$ $(1 <= n, m <= 1000)$.
      ],
      output:
      [
        Выведите одно целое число — сумму $n + m$.
      ],
      (
        "3 4",
        "7",
      ),
      subgroups_scoring:
      (
        "0", "Тесты из условия", "0", "—",
        "1", $(1 <= n, m <= 500)$, "50", "0",
        "2", $(1 <= n, m <= 1000)$, "50", "—",
      ),
    )

    #pagebreak()

    #problem(
      "E. Сумма чисел V",
      "1 секунда",
      "256 мегабайт",
      [
        *Это задача с потестовой оценкой.*\
        Даны целые числа $n$ и $m$. Вычислите их сумму.
      ],
      input:
      [
        В единственной строке записаны целые числа $n$ и $m$ $(1 <= n, m <= 1000)$.
      ],
      output:
      [
        Выведите одно целое число — сумму $n + m$.
      ],
      (
        "3 4",
        "7",
      ),
      per_test_scoring:
      [
        *В этой задаче два теста помимо тестов из условия; каждый из них оценивается независимо в 50 баллов.*
      ],
    )

    #pagebreak()

    #problem(
      "F. Угадай число",
      "1 секунда",
      "256 мегабайт",
      [
        *Это интерактивная задача.*\
        Загадано целое число от $1$ до $t$, где $t$ — натуральное число, сообщаемое программе. Разрешается выполнить не более $k$ запросов вида `? x` ($k$ — скрытое число):
        - если $x$ равно загаданному числу, интерактор отвечает `=`, и программа должна немедленно завершиться;
        - если загаданное число меньше $x$, интерактор отвечает `<`;
        - иначе интерактор отвечает `>`.
      ],
      interactive_protocol:
      [
        Сначала интерактор сообщает натуральное число $t$ $(1 <= t <= 10000)$ — верхнюю границу загаданного числа.\
        Затем программа может выполнить не более $t$ запросов вида `? x`, где $x$ — натуральное число от $1$ до $t$. На каждый запрос интерактор отвечает одним из символов `<`, `>` или `=`.\
        После получения ответа `=` программа обязана немедленно завершиться.
      ],
      (
        "10\n>\n<\n=",
        "\n? 5\n? 8\n? 7",
      ),
      subgroups_scoring:
      (
        "0", "Тесты из условия", "0", "—",
        "1", $(1 <= t <= 10000)$, "100", "0",
      ),
    )

    #pagebreak()

    #problem(
      "G. Прибавь 1",
      "1 секунда",
      "256 мегабайт",
      [
        *Это задача с двойным запуском.*\
        Программа запускается дважды. В первом запуске она получает целое число $n$ и должна вывести $n + 1$. Второй запуск получает на вход значение, выведенное первым запуском, и должен восстановить исходное число $n$. Иных данных о $n$ второй запуск не получает.
      ],
      first_run:
      [
        В единственной строке записано целое число $n$ $(1 <= n <= 1000)$. Выведите $n + 1$.
      ],
      second_run:
      [
        В единственной строке записано целое число $v$ $(2 <= v <= 1001)$ — результат первого запуска. Выведите $v - 1$.
      ],
      (
        "0\n10",
        "11",
        "1\n11",
        "10",
      ),
      comments:
      [
        В примере приведены оба запуска по отдельности.
      ],
      subgroups_scoring:
      (
        "0", "Тесты из условия", "0", "—",
        "1", $(1 <= n <= 1000)$, "100", "0",
      ),
    )
  ]
)
