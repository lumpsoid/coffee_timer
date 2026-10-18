import 'package:coffee_timer/holders/boot_holder.dart';
import 'package:coffee_timer/holders/brew_holder.dart';
import 'package:coffee_timer/holders/editor_holder.dart';
import 'package:coffee_timer/holders/settings_holder.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:flutter_test/flutter_test.dart';

import '../fixtures.dart';

void main() {
  group('BrewHolder', () {
    test('each update replaces the state', () {
      final holder = BrewHolder();
      expect(holder.state, isNull);

      final idle = BrewUpdateIdle(recipe: brewRecipe());
      holder.apply(idle);
      expect(holder.state, idle);

      final running = BrewUpdateRunning(
        recipe: brewRecipe(
          statuses: [BrewStepStatus.current, BrewStepStatus.pending],
        ),
        timer: const BrewTimer(
          run: 1,
          stepIndex: 0,
          stepName: 'Step 1',
          positionLabel: 'Step 1 of 2',
          remainingLabel: '00:15',
          nextLabel: 'Next: Step 2 · 0:15',
          sweepSeconds: 15,
        ),
      );
      holder.apply(running);
      expect(holder.state, running);

      final finished = BrewUpdateFinished(
        recipe: brewRecipe(
          statuses: [BrewStepStatus.done, BrewStepStatus.done],
        ),
        finished: const BrewFinished(
          totalLabel: '0:30',
          pouredLabel: 'Poured to 250g',
        ),
      );
      holder.apply(finished);
      expect(holder.state, finished);

      const empty = BrewUpdateEmpty(label: 'Empty recipe');
      holder.apply(empty);
      expect(holder.state, empty);
    });
  });

  group('EditorHolder', () {
    test('each update replaces the state', () {
      final holder = EditorHolder();
      expect(holder.state, isNull);

      final open = EditorUpdateOpen(draft: draft());
      holder.apply(open);
      expect(holder.state, open);

      holder.apply(const EditorUpdateClosed());
      expect(holder.state, isA<EditorUpdateClosed>());
    });

    test('two identical notices notify twice', () {
      final holder = EditorHolder();
      final notices = <String?>[];
      holder
        ..addListener(() {
          final state = holder.state;
          if (state is EditorUpdateOpen) notices.add(state.draft.notice);
        })
        ..apply(EditorUpdateOpen(draft: draft(notice: 'Failed to save')))
        ..apply(EditorUpdateOpen(draft: draft(notice: 'Failed to save')));

      expect(notices, ['Failed to save', 'Failed to save']);
    });

    test('a Closed notifies even when already closed', () {
      final holder = EditorHolder();
      var notifications = 0;
      holder
        ..addListener(() => notifications++)
        ..apply(const EditorUpdateClosed())
        ..apply(const EditorUpdateClosed());

      expect(notifications, 2);
      expect(holder.state, isA<EditorUpdateClosed>());
    });
  });

  group('SettingsHolder', () {
    test('each update replaces the state', () {
      final holder = SettingsHolder();
      expect(holder.state, isNull);

      final dark = settings(theme: ThemeChoice.dark);
      holder.apply(dark);
      expect(holder.state, dark);

      final red = settings(accent: 0xFFF44336);
      holder.apply(red);
      expect(holder.state, red);
    });
  });

  group('BootHolder', () {
    test('a failure is shown until the next attempt', () {
      final holder = BootHolder()..fail('no such directory');
      expect(holder.failure, 'no such directory');
    });
  });
}
