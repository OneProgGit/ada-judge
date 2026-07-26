#let lang = "en"
#let tr = (
  en: (
    page: "1 of 1",
  ),
  ru: (
    page: "Страница 1 из 1",
  )
)
#let t(key) = tr.at(lang).at(key)
#let template(title, subtitle: none, body) = {
  set page(
    paper: "a5",
    margin: (x: 20pt, top: 130pt, bottom: 50pt),
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
#let problem(title, contents) = {
  [
    = #title
    #contents $square.filled$
  ]
}

#template([Editorial for the Test Contest], subtitle: [long, long ago],
  [
    #problem("A. Sum of Numbers I",
      [
        For all his prudence, the merchant Aristarkh Pankratyevich has set a simple task: observe that the answer is simply the number $n + m$. No further reasoning is required -- read $n$ and $m$, add them, and print the result. Under the given constraints $(1 <= n, m <= 1000)$ the sum is certain to fit in any integer type, so overflow need not concern us. The solution runs in $O(1)$.
      ]
    )

    #pagebreak()

    #problem("B. Sum of Numbers II",
      [
        The solution is identical to that of problem A: the answer is $n + m$. The split into subgroups here serves a purely didactic purpose -- to remind the reader that not every subgroup makes the algorithm harder; sometimes it merely narrows the range of the input while leaving the solution unchanged. A correct solution for the first subgroup is therefore automatically correct for the second as well, and writing separate code for $(1 <= n, m <= 500)$ would be a needless expense of ink.
      ]
    )

    #pagebreak()

    #problem("C. Sum of Numbers III",
      [
        For all his forgetfulness, the station keeper has set the very same task: the answer is the sum $n + m$. Full marks are awarded for passing all the tests of subgroup 1, which include the sample tests as well. The problem holds no further tricks -- it suffices to read two numbers and print their sum.
      ]
    )

    #pagebreak()

    #problem("D. Sum of Numbers IV",
      [
        Algorithmically the problem does not differ from the previous three: the answer is $n + m$. What deserves attention is only the merged-subgroup scoring: to receive the points for subgroup 2, a solution must pass *all* the tests of both subgroup 1 and subgroup 2 -- in other words, the points for subgroup 2 are not awarded on their own, but only in addition to an already-passed subgroup 1. Since the same solution works for every constraint, a correct submission receives the full hundred points at once.
      ]
    )

    #pagebreak()

    #problem("E. Sum of Numbers V",
      [
        Here too the answer is $n + m$, read and printed without any special contrivance. The only difference from problems A--D lies in the scoring: each of the two additional tests, beyond the sample tests, is scored independently (50 points apiece) rather than as a single subgroup. In practice this makes no difference to the solver -- a partial solution is simply impossible here, since the problem is either solved in full or not solved at all.
      ]
    )

    #pagebreak()

    #problem("F. Guess the Number",
      [
        Cornet Sven offers a classic exercise in *binary search*. Maintain the bounds of the interval of possible values $["lo", "hi"]$, initially $"lo" = 1$, $"hi" = t$. At each step query the midpoint $x = floor(("lo" + "hi") / 2)$:
        - on the reply `=` -- the number has been guessed; terminate at once;
        - on the reply `<` (the number sought is less than $x$) -- shrink the interval to $["lo", x - 1]$;
        - on the reply `>` -- shrink the interval to $[x + 1, "hi"]$.

        The interval shrinks by at least half with every query, so the number of queries never exceeds $ceil(log_2 t)$. For $t <= 10000$, fourteen queries suffice -- comfortably within any reasonable hidden bound $k$. Remember to flush the output after every query, and to terminate the program the moment the reply `=` is received.
      ]
    )

    #pagebreak()

    #problem("G. Add 1",
      [
        The problem is best understood by splitting it into the roles of the clerk and the official. In the first run, the program reads a number $n$ and prints $n + 1$ -- nothing more complicated than adding one is required here. In the second run, the program receives on its input exactly the number printed by the first run (the judge itself carries this value from the first run's standard output to the second run's standard input -- no temporary files need be created), and it must print $v - 1$, that is, the original $n$.

        The chief difficulty of the problem is not algorithmic but organizational: one must ensure that both runs read and write through the standard streams exactly once, with no extraneous prompts or debug output that might confuse the judging system.
      ]
    )
  ]
)
