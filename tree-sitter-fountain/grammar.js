module.exports = grammar({
  name: 'fountain',
  
  conflicts: $ => [
    [$.action, $.dialogue, $.character_line],
  ],
  
  rules: {
    document: $ => repeat1($._line),
    
    _line: $ => choice(
      $.title_page_line,
      $.section,
      $.synopsis,
      $.scene_heading,
      $.transition,
      $.centered,
      $.character_line,
      $.parenthetical,
      $.dialogue,
      $.page_break,
      $.lyrics,
      $.note_inline,
      $.boneyard,
      $.separator,
      $.action,
    ),
    
    title_page_line: $ => seq('Title:', /[^\n]*/),
    
    section: $ => seq('#', $.word),
    synopsis: $ => seq('=', /[^\n]*/),
    
    scene_heading: $ => choice(
      seq('INT.', $.scene_location),
      seq('EXT.', $.scene_location),
      seq('EST.', $.scene_location),
      seq('INT./EXT.', $.scene_location),
      seq('.', $.scene_location),
    ),
    scene_location: $ => $.word,
    
    transition: $ => seq('>', $.word),
    centered: $ => seq('>', $.word, '<'),
    page_break: $ => '===',
    separator: $ => '---',
    lyrics: $ => seq('~', $.word),
    
    character_line: $ => seq($.word, optional(seq('(', $.word, ')'))),
    parenthetical: $ => seq('(', $.word, ')'),
    dialogue: $ => $.word,
    note_inline: $ => seq('[[', $.word, ']]'),
    boneyard: $ => seq('/*', $.word, '*/'),
    action: $ => $.word,
    word: $ => /[^\n]+/,
  },
  
  extras: $ => [/[\t ]+/],
});