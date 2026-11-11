%%% Rust-Edu Workshop format.
%%% Bart Massey 2022-07-20

\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{rust-edu-workshop}
\LoadClass{article}
\RequirePackage{url}

%%% Use Helvetica because good lookin'.
\RequirePackage{helvet}
\renewcommand\familydefault{\sfdefault}

%%% Reduce margins because online.
\oddsidemargin=-0.25in
\evensidemargin=\oddsidemargin
\textwidth=7in
\addtolength{\textwidth}{-2em}
\topmargin=-0.25in
\headheight=\baselineskip
\headsep=\baselineskip
\textheight=9.5in
\addtolength{\textheight}{-2\baselineskip}

%%% Turn off page numbering because collections.
\pagestyle{empty}
\let\@@maketitle=\maketitle
\def\maketitle{\@@maketitle\thispagestyle{empty}}

%%% Use block paragraphs.
\parskip=0.5\baselineskip
\parindent=0pt
\renewenvironment{quotation}
               {\list{}{\listparindent 0pt%
                        \itemindent    \listparindent
                        \rightmargin   \leftmargin
                        \parsep        \parskip}%
                \item\relax}
               {\endlist}

%%% Title in block style.
%%% Authors left.
\DeclareRobustCommand\and{%   % \begin{tabular}
  \end{tabular}%
  \hskip 1em \@plus.17fil%
  \begin{tabular}[t]{@{}l}}%     % \end{tabular}
\def\@maketitle{%
  \clearpage
  \null
  \let \footnote \thanks
  {\raggedright
    {\large \bfseries \@title}\\*[1.5ex]
    {\begin{tabular}[t]{@{}l}\@author\end{tabular}}\\*[1.5ex]
    {\bfseries Rust-Edu Workshop \@date}\\
    {\url{https://rust-edu.org/workshop}}}%
  \par \vskip 1.5em}

%%% Section headers shrunk to match title size.
\RequirePackage{titlesec}
\titleformat*{\section}{\large\bfseries}
\titleformat*{\subsection}{\large\bfseries}
\titleformat*{\subsubsection}{\large\bfseries}
\titleformat*{\paragraph}{\bfseries}
\titleformat*{\subparagraph}{\bfseries}

%%% Format email addresses reasonably.
\newcommand{\email}[1]{{\tt <{#1}>}}
